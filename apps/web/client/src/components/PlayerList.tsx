import { useState } from 'react'
import './PlayerList.css'

interface Player {
  identity: string
  name: string
  x: number
  y: number
  z: number
  rotationY: number
  online: boolean
}

interface PlayerListProps {
  players: Player[]
  identity: string | null
  setName: (name: string) => void
}

export function PlayerList({ players, identity, setName }: PlayerListProps) {
  const [isEditing, setIsEditing] = useState(false)
  const [editName, setEditName] = useState('')

  const currentPlayer = players.find(p => p.identity === identity)

  const handleStartEdit = () => {
    setEditName(currentPlayer?.name || '')
    setIsEditing(true)
  }

  const handleSave = () => {
    if (editName.trim()) {
      setName(editName.trim())
    }
    setIsEditing(false)
  }

  return (
    <div className="player-list">
      <h3>Players</h3>
      <ul>
        {players.filter(p => p.online).map(player => (
          <li key={player.identity} className={player.identity === identity ? 'self' : ''}>
            {player.identity === identity && isEditing ? (
              <div className="name-edit">
                <input
                  type="text"
                  value={editName}
                  onChange={e => setEditName(e.target.value)}
                  maxLength={32}
                  autoFocus
                />
                <button onClick={handleSave}>✓</button>
              </div>
            ) : (
              <>
                <span className="status-dot" />
                <span className="name">{player.name}</span>
                {player.identity === identity && (
                  <button className="edit-btn" onClick={handleStartEdit}>✎</button>
                )}
              </>
            )}
          </li>
        ))}
      </ul>
    </div>
  )
}
