import { useRef } from 'react'
import { Canvas, useFrame } from '@react-three/fiber'
import { OrbitControls, Grid, PerspectiveCamera } from '@react-three/drei'
import * as THREE from 'three'

interface WorldEntity {
  id: number
  entityType: string
  x: number
  y: number
  z: number
  rotationY: number
  scale: number
  metadata: string
}

interface Player {
  identity: string
  name: string
  x: number
  y: number
  z: number
  rotationY: number
  online: boolean
}

interface World3DProps {
  entities: WorldEntity[]
  players: Player[]
  onMove: (x: number, y: number, z: number, rotationY: number) => void
}

function Entity({ entity }: { entity: WorldEntity }) {
  const ref = useRef<THREE.Mesh>(null)

  // Get color based on entity type
  const getColor = () => {
    switch (entity.entityType) {
      case 'tree': return '#228B22'
      case 'rock': return '#808080'
      case 'building': return '#8B4513'
      case 'water': return '#4169E1'
      default: return '#FF69B4'
    }
  }

  // Get geometry based on entity type
  const renderGeometry = () => {
    switch (entity.entityType) {
      case 'tree':
        return (
          <group>
            <mesh position={[0, 0.5, 0]}>
              <cylinderGeometry args={[0.2, 0.3, 1, 8]} />
              <meshStandardMaterial color="#8B4513" />
            </mesh>
            <mesh position={[0, 1.5, 0]}>
              <coneGeometry args={[1, 2, 8]} />
              <meshStandardMaterial color={getColor()} />
            </mesh>
          </group>
        )
      case 'rock':
        return (
          <mesh>
            <dodecahedronGeometry args={[0.5]} />
            <meshStandardMaterial color={getColor()} flatShading />
          </mesh>
        )
      case 'building':
        return (
          <group>
            <mesh position={[0, 1, 0]}>
              <boxGeometry args={[2, 2, 2]} />
              <meshStandardMaterial color={getColor()} />
            </mesh>
            <mesh position={[0, 2.5, 0]} rotation={[0, Math.PI / 4, 0]}>
              <coneGeometry args={[1.5, 1, 4]} />
              <meshStandardMaterial color="#A0522D" />
            </mesh>
          </group>
        )
      default:
        return (
          <mesh>
            <boxGeometry args={[1, 1, 1]} />
            <meshStandardMaterial color={getColor()} />
          </mesh>
        )
    }
  }

  return (
    <group
      ref={ref}
      position={[entity.x, entity.y, entity.z]}
      rotation={[0, (entity.rotationY * Math.PI) / 180, 0]}
      scale={entity.scale}
    >
      {renderGeometry()}
    </group>
  )
}

function PlayerAvatar({ player, isSelf }: { player: Player; isSelf: boolean }) {
  const ref = useRef<THREE.Group>(null)

  useFrame(() => {
    if (ref.current) {
      // Smooth interpolation to target position
      ref.current.position.x = THREE.MathUtils.lerp(ref.current.position.x, player.x, 0.1)
      ref.current.position.y = THREE.MathUtils.lerp(ref.current.position.y, player.y, 0.1)
      ref.current.position.z = THREE.MathUtils.lerp(ref.current.position.z, player.z, 0.1)
    }
  })

  return (
    <group ref={ref} position={[player.x, player.y, player.z]}>
      {/* Body */}
      <mesh position={[0, 0.5, 0]}>
        <capsuleGeometry args={[0.3, 0.6, 4, 8]} />
        <meshStandardMaterial color={isSelf ? '#4F9' : '#4AF'} />
      </mesh>
      {/* Head */}
      <mesh position={[0, 1.2, 0]}>
        <sphereGeometry args={[0.25, 16, 16]} />
        <meshStandardMaterial color={isSelf ? '#4F9' : '#4AF'} />
      </mesh>
      {/* Name label */}
      <sprite position={[0, 1.8, 0]}>
        <textGeometry args={[player.name, { size: 0.2, height: 0.02 }]} />
      </sprite>
    </group>
  )
}

function Ground() {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.01, 0]}>
      <planeGeometry args={[100, 100]} />
      <meshStandardMaterial color="#3a5f3a" />
    </mesh>
  )
}

function Scene({ entities, players, identity }: { entities: WorldEntity[]; players: Player[]; identity: string | null }) {
  return (
    <>
      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 20, 10]} intensity={1} castShadow />
      <pointLight position={[-10, 10, -10]} intensity={0.5} />

      <Ground />
      <Grid args={[100, 100]} position={[0, 0.01, 0]} />

      {entities.map(entity => (
        <Entity key={entity.id} entity={entity} />
      ))}

      {players.filter(p => p.online).map(player => (
        <PlayerAvatar
          key={player.identity}
          player={player}
          isSelf={player.identity === identity}
        />
      ))}

      <OrbitControls
        target={[0, 0, 0]}
        maxPolarAngle={Math.PI / 2 - 0.1}
        minDistance={5}
        maxDistance={50}
      />

      <PerspectiveCamera makeDefault position={[20, 15, 20]} />
    </>
  )
}

export function World3D({ entities, players, onMove }: World3DProps) {
  return (
    <Canvas shadows>
      <Scene entities={entities} players={players} identity={null} />
    </Canvas>
  )
}
