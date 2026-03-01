import { useRef, useMemo } from 'react'
import { Canvas, useFrame } from '@react-three/fiber'
import { OrbitControls, Grid, PerspectiveCamera, Text } from '@react-three/drei'
import * as THREE from 'three'

// Types from useSpacetime hook (flat fields for SpacetimeDB 2.0)
interface WorldEntityRow {
  id: number
  name: string
  entityType: string
  x: number
  y: number
  z: number
  rotPitch: number
  rotYaw: number
  rotRoll: number
  scale: number
  shapeJson: string
  materialJson: string | null
  lightJson: string | null
  behaviorsJson: string
  audioJson: string | null
  chunkX: number
  chunkY: number
  owner: string | null
  createdAt: bigint
}

interface Player {
  identity: string
  device: string
  name: string
  x: number
  y: number
  z: number
  rotationY: number
  online: boolean
  lastSeen: bigint
}

interface World3DProps {
  entities: WorldEntityRow[]
  players: Player[]
  onMove: (x: number, y: number, z: number, rotationY: number) => void
}

// Parse shape from JSON
function parseShape(shapeJson: string): { kind: string; params: Record<string, number> } {
  try {
    const parsed = JSON.parse(shapeJson)
    const key = Object.keys(parsed)[0]
    if (key) {
      return { kind: key.toLowerCase(), params: parsed[key] }
    }
  } catch {
    // Ignore parse errors
  }
  return { kind: 'cuboid', params: { x: 1, y: 1, z: 1 } }
}

// Parse color from material JSON
function parseMaterialColor(materialJson: string | null): string {
  if (!materialJson) return '#888888'
  try {
    const mat = JSON.parse(materialJson)
    if (mat.color && Array.isArray(mat.color)) {
      const [r, g, b] = mat.color
      return `rgb(${Math.round(r * 255)}, ${Math.round(g * 255)}, ${Math.round(b * 255)})`
    }
  } catch {
    // Ignore
  }
  return '#888888'
}

function Entity({ entity }: { entity: WorldEntityRow }) {
  const ref = useRef<THREE.Group>(null)
  const { kind, params } = parseShape(entity.shapeJson)
  const color = parseMaterialColor(entity.materialJson)

  const geometry = useMemo(() => {
    switch (kind) {
      case 'sphere':
        return <sphereGeometry args={[params.radius || 0.5, 16, 16]} />
      case 'cylinder':
        return <cylinderGeometry args={[params.radius || 0.5, params.radius || 0.5, params.height || 1, 16]} />
      case 'cone':
        return <coneGeometry args={[params.radius || 0.5, params.height || 1, 16]} />
      case 'capsule':
        return <capsuleGeometry args={[params.radius || 0.25, params.half_length || 0.5, 8, 16]} />
      case 'torus':
        return <torusGeometry args={[params.major_radius || 1, params.minor_radius || 0.25, 16, 32]} />
      case 'plane':
        return <boxGeometry args={[params.x || 10, 0.1, params.z || 10]} />
      case 'cuboid':
      default:
        return <boxGeometry args={[params.x || 1, params.y || 1, params.z || 1]} />
    }
  }, [kind, params])

  // Convert rotation from degrees to radians
  const rotation = useMemo(() => [
    (entity.rotPitch * Math.PI) / 180,
    (entity.rotYaw * Math.PI) / 180,
    (entity.rotRoll * Math.PI) / 180,
  ], [entity.rotPitch, entity.rotYaw, entity.rotRoll])

  return (
    <group
      ref={ref}
      position={[entity.x, entity.y, entity.z]}
      rotation={rotation}
      scale={entity.scale}
    >
      <mesh castShadow receiveShadow>
        {geometry}
        <meshStandardMaterial color={color} />
      </mesh>

      {/* Entity name label */}
      <Text
        position={[0, 1.5, 0]}
        fontSize={0.3}
        color="white"
        anchorX="center"
        anchorY="middle"
      >
        {entity.name}
      </Text>
    </group>
  )
}

function PlayerAvatar({ player, isSelf }: { player: Player; isSelf: boolean }) {
  const ref = useRef<THREE.Group>(null)

  useFrame(() => {
    if (ref.current) {
      ref.current.position.x = THREE.MathUtils.lerp(ref.current.position.x, player.x, 0.1)
      ref.current.position.y = THREE.MathUtils.lerp(ref.current.position.y, player.y, 0.1)
      ref.current.position.z = THREE.MathUtils.lerp(ref.current.position.z, player.z, 0.1)
    }
  })

  return (
    <group ref={ref} position={[player.x, player.y, player.z]}>
      {/* Body */}
      <mesh position={[0, 0.5, 0]} castShadow>
        <capsuleGeometry args={[0.3, 0.6, 4, 8]} />
        <meshStandardMaterial color={isSelf ? '#4F9' : '#4AF'} />
      </mesh>
      {/* Head */}
      <mesh position={[0, 1.2, 0]} castShadow>
        <sphereGeometry args={[0.25, 16, 16]} />
        <meshStandardMaterial color={isSelf ? '#4F9' : '#4AF'} />
      </mesh>
      {/* Name label */}
      <Text
        position={[0, 1.8, 0]}
        fontSize={0.25}
        color={isSelf ? '#4F9' : '#4AF'}
        anchorX="center"
        anchorY="middle"
      >
        {player.name}
      </Text>
    </group>
  )
}

function Ground() {
  return (
    <mesh rotation={[-Math.PI / 2, 0, 0]} position={[0, -0.01, 0]} receiveShadow>
      <planeGeometry args={[200, 200]} />
      <meshStandardMaterial color="#3a5f3a" />
    </mesh>
  )
}

function Scene({ entities, players, identity }: { entities: WorldEntityRow[]; players: Player[]; identity: string | null }) {
  return (
    <>
      <ambientLight intensity={0.5} />
      <directionalLight position={[10, 20, 10]} intensity={1} castShadow />
      <pointLight position={[-10, 10, -10]} intensity={0.5} />

      <Ground />
      <Grid args={[200, 200]} position={[0, 0.01, 0]} cellSize={64} fadeDistance={150} />

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
        maxDistance={100}
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
