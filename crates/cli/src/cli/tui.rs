use anyhow::Result;
use clap::Args;
use crossterm::{
    event::{Event, KeyCode, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use futures::StreamExt;
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use std::io;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

use localgpt_core::agent::{
    Agent, AgentConfig, create_spawn_agent_tool, get_last_session_id_for_agent,
};
use localgpt_core::concurrency::WorkspaceLock;
use localgpt_core::config::Config;
use localgpt_core::memory::MemoryManager;

#[derive(Args)]
pub struct TuiArgs {
    /// Model to use (overrides config)
    #[arg(short, long)]
    pub model: Option<String>,

    /// Session ID to resume
    #[arg(short, long)]
    pub session: Option<String>,

    /// Resume the most recent session
    #[arg(long)]
    pub resume: bool,
}

pub async fn run(args: TuiArgs, agent_id: &str) -> Result<()> {
    // 1. Setup Agent
    let config = Config::load()?;
    let memory = Arc::new(MemoryManager::new_with_full_config(
        &config.memory,
        Some(&config),
        agent_id,
    )?);

    let agent_config = AgentConfig {
        model: args.model.unwrap_or(config.agent.default_model.clone()),
        context_window: config.agent.context_window,
        reserve_tokens: config.agent.reserve_tokens,
    };

    let mut agent = Agent::new(agent_config, &config, Arc::clone(&memory)).await?;
    agent.extend_tools(localgpt_cli_tools::create_cli_tools(&config)?);
    agent.extend_tools(vec![create_spawn_agent_tool(config.clone(), memory)]);
    debug!("TUI Agent initialized with tools: {:?}", agent.tool_names());

    let _workspace_lock = WorkspaceLock::new()?;

    // Load or create session
    let session_id = if let Some(id) = args.session {
        Some(id)
    } else if args.resume {
        get_last_session_id_for_agent(agent_id)?
    } else {
        None
    };

    if let Some(session_id) = session_id {
        if let Err(e) = agent.resume_session(&session_id).await {
            eprintln!("Failed to load session {}: {}", session_id, e);
            agent.new_session().await?;
        }
    } else {
        agent.new_session().await?;
    }

    // 2. Setup Terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // 3. Run App
    let app_result = run_app(&mut terminal, &mut agent, agent_id).await;

    // 4. Restore Terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if let Err(e) = app_result {
        eprintln!("TUI Error: {:?}", e);
    } else if let Err(e) = agent.auto_save_session() {
        eprintln!("Warning: Failed to auto-save session: {}", e);
    }

    Ok(())
}

#[derive(Clone)]
enum AppMessage {
    Text {
        role: String,
        content: String,
    },
    ToolCall {
        name: String,
        arguments: String,
        output: Option<String>,
        is_expanded: bool,
    },
}

struct App {
    input: String,
    messages: Vec<AppMessage>,
    is_generating: bool,
    selected_index: Option<usize>,
}

impl App {
    fn new(agent_id: &str, model: &str, chunk_count: usize, has_embeddings: bool) -> Self {
        let embedding_status = if has_embeddings {
            " | Embeddings: enabled"
        } else {
            ""
        };

        let header = format!(
            "LocalGPT v{} | Agent: {} | Model: {} | Memory: {} chunks{}",
            env!("CARGO_PKG_VERSION"),
            agent_id,
            model,
            chunk_count,
            embedding_status
        );

        Self {
            input: String::new(),
            messages: vec![
                AppMessage::Text {
                    role: "System".to_string(),
                    content: header,
                },
                AppMessage::Text {
                    role: "System".to_string(),
                    content: "Use Up/Down to select, Ctrl+O to expand tools. Esc/Ctrl+C to quit."
                        .to_string(),
                },
            ],
            is_generating: false,
            selected_index: None,
        }
    }
}

enum AppEvent {
    Input(Event),
    Tick,
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    agent: &mut Agent,
    agent_id: &str,
) -> Result<()> {
    let mut app = App::new(
        agent_id,
        agent.model(),
        agent.memory_chunk_count(),
        agent.has_embeddings(),
    );

    // Load history messages if resuming (use raw_session_messages to avoid printing the appended security block)
    let mut pending_tool_outputs: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();

    for msg in agent.raw_session_messages() {
        if msg.message.role == localgpt_core::agent::Role::System {
            // Skip the huge internal system prompt from showing up in UI
            continue;
        }

        if msg.message.role == localgpt_core::agent::Role::Tool {
            if let Some(ref id) = msg.message.tool_call_id {
                pending_tool_outputs.insert(id.clone(), msg.message.content.clone());
            }
            continue;
        }

        let role = match msg.message.role {
            localgpt_core::agent::Role::User => "You",
            localgpt_core::agent::Role::Assistant => "Assistant",
            localgpt_core::agent::Role::System => "System",
            localgpt_core::agent::Role::Tool => "Tool",
        };

        if let Some(ref calls) = msg.message.tool_calls {
            for call in calls {
                app.messages.push(AppMessage::ToolCall {
                    name: call.name.clone(),
                    arguments: call.arguments.clone(),
                    output: pending_tool_outputs.remove(&call.id), // Might be None if not yet processed or tracked differently in history
                    is_expanded: false,
                });
            }
        }

        if !msg.message.content.is_empty() {
            app.messages.push(AppMessage::Text {
                role: role.to_string(),
                content: msg.message.content.clone(),
            });
        }
    }

    // Now re-iterate the app messages. In LocalGPT's history, Assistant messages emit tool_calls,
    // and then the *next* messages are Role::Tool with the outputs.
    // Let's patch those historical outputs in.
    let raw_msgs = agent.raw_session_messages();
    for i in 0..app.messages.len() {
        if let AppMessage::ToolCall {
            ref name,
            ref arguments,
            ref mut output,
            ..
        } = app.messages[i]
            && output.is_none()
        {
            // Find matching tool call in raw history to get ID
            if let Some(msg) = raw_msgs.iter().find(|m| {
                m.message.tool_calls.as_ref().is_some_and(|calls| {
                    calls
                        .iter()
                        .any(|c| &c.name == name && &c.arguments == arguments)
                })
            }) && let Some(calls) = &msg.message.tool_calls
                && let Some(call) = calls
                    .iter()
                    .find(|c| &c.name == name && &c.arguments == arguments)
            {
                // Find subsequent Role::Tool message with this ID
                if let Some(tool_msg) = raw_msgs.iter().find(|m| {
                    m.message.role == localgpt_core::agent::Role::Tool
                        && m.message.tool_call_id.as_ref() == Some(&call.id)
                }) {
                    *output = Some(tool_msg.message.content.clone());
                }
            }
        }
    }

    app.selected_index = if app.messages.is_empty() {
        None
    } else {
        Some(app.messages.len() - 1)
    };

    let (tx, mut rx) = mpsc::channel(100);

    // Input thread
    let tick_rate = std::time::Duration::from_millis(50);
    let _input_task = tokio::spawn(async move {
        loop {
            if crossterm::event::poll(tick_rate).unwrap_or(false)
                && let Ok(evt) = crossterm::event::read()
            {
                let _ = tx.send(AppEvent::Input(evt)).await;
            }
            let _ = tx.send(AppEvent::Tick).await;
        }
    });

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if let Some(event) = rx.recv().await {
            match event {
                AppEvent::Input(Event::Key(key)) if key.kind == KeyEventKind::Press => {
                    // Check for Ctrl+C to quit
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('c')
                    {
                        return Ok(());
                    }

                    // Check for Ctrl+O
                    if key.modifiers.contains(KeyModifiers::CONTROL)
                        && key.code == KeyCode::Char('o')
                    {
                        if let Some(idx) = app.selected_index
                            && let Some(AppMessage::ToolCall { is_expanded, .. }) =
                                app.messages.get_mut(idx)
                        {
                            *is_expanded = !*is_expanded;
                        }
                        continue;
                    }

                    match key.code {
                        KeyCode::Esc => {
                            return Ok(());
                        }
                        KeyCode::Up => {
                            if let Some(idx) = app.selected_index
                                && idx > 0
                            {
                                app.selected_index = Some(idx - 1);
                            } else if !app.messages.is_empty() {
                                app.selected_index = Some(app.messages.len() - 1);
                            }
                        }
                        KeyCode::Down => {
                            if let Some(idx) = app.selected_index
                                && idx + 1 < app.messages.len()
                            {
                                app.selected_index = Some(idx + 1);
                            }
                        }
                        KeyCode::Enter => {
                            if app.is_generating {
                                continue;
                            }
                            let input = app.input.trim().to_string();
                            app.input.clear();

                            if !input.is_empty() {
                                app.messages.push(AppMessage::Text {
                                    role: "You".to_string(),
                                    content: input.clone(),
                                });
                                app.is_generating = true;
                                // Placeholder for assistant message
                                app.messages.push(AppMessage::Text {
                                    role: "Assistant".to_string(),
                                    content: String::new(),
                                });
                                app.selected_index = Some(app.messages.len() - 1);
                                terminal.draw(|f| ui(f, &app))?;

                                // Stream response
                                match agent.chat_stream_with_tools(&input, Vec::new()).await {
                                    Ok(stream) => {
                                        tokio::pin!(stream);
                                        while let Some(evt) = stream.next().await {
                                            match evt {
                                                Ok(localgpt_core::agent::StreamEvent::Content(text)) => {
                                                    if let Some(AppMessage::Text { content, .. }) = app.messages.last_mut() {
                                                        content.push_str(&text);
                                                    } else {
                                                        app.messages.push(AppMessage::Text { role: "Assistant".to_string(), content: text });
                                                        app.selected_index = Some(app.messages.len() - 1);
                                                    }
                                                    terminal.draw(|f| ui(f, &app))?;
                                                }
                                                Ok(localgpt_core::agent::StreamEvent::ToolCallStart { name, arguments, .. }) => {
                                                    app.messages.push(AppMessage::ToolCall {
                                                        name: name.clone(),
                                                        arguments: arguments.to_string(),
                                                        output: None,
                                                        is_expanded: false,
                                                    });
                                                    app.selected_index = Some(app.messages.len() - 1);
                                                    terminal.draw(|f| ui(f, &app))?;
                                                }
                                                Ok(localgpt_core::agent::StreamEvent::ToolCallEnd { name: _, output, .. }) => {
                                                    if let Some(AppMessage::ToolCall { output: out, .. }) = app.messages.last_mut() {
                                                        *out = Some(output);
                                                    }
                                                    terminal.draw(|f| ui(f, &app))?;
                                                }
                                                Ok(localgpt_core::agent::StreamEvent::Done) => {}
                                                Err(e) => {
                                                    app.messages.push(AppMessage::Text { role: "System".to_string(), content: format!("[Error: {}]", e) });
                                                    terminal.draw(|f| ui(f, &app))?;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        app.messages.push(AppMessage::Text {
                                            role: "System".to_string(),
                                            content: format!("Error starting stream: {}", e),
                                        });
                                    }
                                }
                                app.is_generating = false;
                                app.selected_index = Some(app.messages.len() - 1);
                            }
                        }
                        KeyCode::Char(c) => {
                            if !app.is_generating {
                                app.input.push(c);
                            }
                        }
                        KeyCode::Backspace => {
                            if !app.is_generating {
                                app.input.pop();
                            }
                        }
                        _ => {}
                    }
                }
                _ => {} // Tick or other events
            }
        }
    }
}

fn ui(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(f.area());

    let mut lines: Vec<Line> = Vec::new();

    for (i, msg) in app.messages.iter().enumerate() {
        let is_selected = app.selected_index == Some(i);
        let prefix = if is_selected { "> " } else { "  " };
        let style = if is_selected {
            Style::default().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        match msg {
            AppMessage::Text { role, content } => {
                let role_color = match role.as_str() {
                    "You" => Color::Cyan,
                    "Assistant" => Color::Green,
                    "System" => Color::DarkGray,
                    _ => Color::Reset,
                };

                // Very basic wrapping
                let mut first = true;
                for line in content.split('\n') {
                    if first {
                        lines.push(Line::from(vec![
                            Span::styled(prefix, style),
                            Span::styled(
                                format!("{}: ", role),
                                Style::default().fg(role_color).add_modifier(Modifier::BOLD),
                            ),
                            Span::raw(line),
                        ]));
                        first = false;
                    } else {
                        lines.push(Line::from(vec![
                            Span::styled("  ", style),
                            Span::raw(format!("  {}", line)),
                        ]));
                    }
                }
            }
            AppMessage::ToolCall {
                name,
                arguments,
                output,
                is_expanded,
            } => {
                let status = if output.is_some() {
                    "[Done]"
                } else {
                    "[Running...]"
                };
                let expand_hint = if *is_expanded { "[-] " } else { "[+] " };

                lines.push(Line::from(vec![
                    Span::styled(prefix, style),
                    Span::styled(expand_hint, Style::default().fg(Color::Yellow)),
                    Span::styled(
                        format!("Tool: {} {}", name, status),
                        Style::default().fg(Color::Magenta),
                    ),
                ]));

                if *is_expanded {
                    lines.push(Line::from(vec![
                        Span::raw("    "),
                        Span::styled("Input: ", Style::default().fg(Color::DarkGray)),
                        Span::raw(arguments.trim()),
                    ]));

                    if let Some(out) = output {
                        lines.push(Line::from(vec![
                            Span::raw("    "),
                            Span::styled("Output: ", Style::default().fg(Color::DarkGray)),
                        ]));
                        for out_line in out.trim().lines() {
                            lines.push(Line::from(vec![Span::raw("      "), Span::raw(out_line)]));
                        }
                    }
                }
            }
        }
    }

    let max_lines = chunks[0].height.saturating_sub(2) as usize;
    let total_lines = lines.len();

    let skip_lines = total_lines.saturating_sub(max_lines);

    let visible_lines = lines.into_iter().skip(skip_lines).collect::<Vec<_>>();

    let title = if app.is_generating {
        " Chat (Generating...) "
    } else {
        " Chat "
    };
    let messages_block =
        Paragraph::new(visible_lines).block(Block::default().title(title).borders(Borders::ALL));
    f.render_widget(messages_block, chunks[0]);

    let input_block = Paragraph::new(app.input.as_str())
        .block(Block::default().title(" Input ").borders(Borders::ALL));
    f.render_widget(input_block, chunks[1]);
}
