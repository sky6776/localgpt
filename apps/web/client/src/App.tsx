import { useState } from 'react'
import { World3D } from './components/World3D'
import { Chat } from './components/Chat'
import { PlayerList } from './components/PlayerList'
import { useSpacetime } from './hooks/useSpacetime'
import './App.css'

function App() {
  const { connected, players, entities, messages, identity, sendMessage, movePlayer, setName } = useSpacetime()
  const [showChat, setShowChat] = useState(true)

  if (!connected) {
    return (
      <div className="loading">
        <div className="spinner" />
        <p>Connecting to world...</p>
      </div>
    )
  }

  return (
    <div className="app">
      <div className="world-container">
        <World3D
          entities={entities}
          players={players}
          onMove={movePlayer}
        />
      </div>

      <div className="ui-overlay">
        <div className="top-bar">
          <h1>LocalGPT World</h1>
          <div className="player-count">
            {players.filter(p => p.online).length} online
          </div>
        </div>

        <PlayerList players={players} identity={identity} setName={setName} />

        <button
          className="chat-toggle"
          onClick={() => setShowChat(!showChat)}
        >
          {showChat ? '▼' : '▲'} Chat
        </button>

        {showChat && (
          <Chat
            messages={messages}
            identity={identity}
            onSend={sendMessage}
          />
        )}
      </div>
    </div>
  )
}

export default App
