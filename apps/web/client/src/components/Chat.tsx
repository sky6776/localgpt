import { useState, useRef, useEffect } from 'react'
import './Chat.css'

interface ChatMessage {
  id: number
  senderIdentity: string
  senderName: string
  message: string
  timestamp: bigint
  msgType: string
}

interface ChatProps {
  messages: ChatMessage[]
  identity: string | null
  onSend: (message: string) => void
}

export function Chat({ messages, identity, onSend }: ChatProps) {
  const [input, setInput] = useState('')
  const listRef = useRef<HTMLDivElement>(null)

  useEffect(() => {
    if (listRef.current) {
      listRef.current.scrollTop = listRef.current.scrollHeight
    }
  }, [messages])

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    if (input.trim()) {
      onSend(input.trim())
      setInput('')
    }
  }

  const formatTime = (timestamp: bigint) => {
    const date = new Date(Number(timestamp) / 1000)
    return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })
  }

  const filteredMessages = messages.filter(m => m.msgType === 'chat' || m.msgType === 'system')

  return (
    <div className="chat">
      <div className="chat-messages" ref={listRef}>
        {filteredMessages.map(msg => (
          <div
            key={msg.id}
            className={`chat-message ${msg.senderIdentity === identity ? 'self' : ''} ${msg.msgType}`}
          >
            {msg.msgType !== 'system' && (
              <>
                <span className="time">{formatTime(msg.timestamp)}</span>
                <span className="sender">{msg.senderName}:</span>
              </>
            )}
            <span className="text">{msg.message}</span>
          </div>
        ))}
      </div>
      <form className="chat-input" onSubmit={handleSubmit}>
        <input
          type="text"
          value={input}
          onChange={e => setInput(e.target.value)}
          placeholder="Type a message..."
          maxLength={500}
        />
        <button type="submit">Send</button>
      </form>
    </div>
  )
}
