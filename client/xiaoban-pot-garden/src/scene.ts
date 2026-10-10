import {
  ACESFilmicToneMapping, CanvasTexture, DirectionalLight, HemisphereLight, Mesh, MeshBasicMaterial, VSMShadowMap, PerspectiveCamera,
  PlaneGeometry, PMREMGenerator, Scene, ShadowMaterial, SRGBColorSpace, WebGLRenderer,
} from 'three'
import { RoomEnvironment } from 'three/examples/jsm/environments/RoomEnvironment.js'

/** Where the camera looks, and how far away it stands: a long lens a little above the pot, as in a product photo. */
export const VIEW = { target: [0, 0.98, 0] as const, distance: 6.0, pitch: 0.3, fov: 24, halfWidth: 1.2 }

function contactShadow(): Mesh {
  const canvas = document.createElement('canvas')
  canvas.width = canvas.height = 256
  const ctx = canvas.getContext('2d')!
  const g = ctx.createRadialGradient(128, 128, 0, 128, 128, 128)
  g.addColorStop(0, 'rgba(90, 55, 30, 0.3)')
  g.addColorStop(0.5, 'rgba(90, 55, 30, 0.14)')
  g.addColorStop(1, 'rgba(90, 55, 30, 0)')
  ctx.fillStyle = g
  ctx.fillRect(0, 0, 256, 256)
  const texture = new CanvasTexture(canvas)
  texture.colorSpace = SRGBColorSpace
  const mesh = new Mesh(new PlaneGeometry(2.5, 2.1), new MeshBasicMaterial({ map: texture, transparent: true, depthWrite: false }))
  mesh.rotation.x = -Math.PI / 2
  mesh.position.y = 0.003
  return mesh
}

export interface Stage {
  scene: Scene
  camera: PerspectiveCamera
  renderer: WebGLRenderer
  resize(): void
  render(): void
  dispose(): void
}

export function createStage(host: HTMLElement): Stage {
  const renderer = new WebGLRenderer({ antialias: true, alpha: true, powerPreference: 'high-performance' })
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2))
  renderer.toneMapping = ACESFilmicToneMapping
  renderer.toneMappingExposure = 1.04
  renderer.shadowMap.enabled = true
  renderer.shadowMap.type = VSMShadowMap
  host.append(renderer.domElement)

  const scene = new Scene()
  const environment = new PMREMGenerator(renderer)
  scene.environment = environment.fromScene(new RoomEnvironment(), 0.04).texture
  scene.environmentIntensity = 0.65
  environment.dispose()

  const key = new DirectionalLight('#fff0df', 2.6)
  key.position.set(-1.7, 6.2, 2.6)
  key.castShadow = true
  key.shadow.mapSize.set(2048, 2048)
  Object.assign(key.shadow.camera, { left: -1.9, right: 1.9, top: 2.6, bottom: -1.4, near: 0.5, far: 14 })
  key.shadow.bias = -0.0004
  key.shadow.normalBias = 0.012
  key.shadow.radius = 14
  key.shadow.blurSamples = 20
  const rim = new DirectionalLight('#dce9ff', 0.9)
  rim.position.set(3.2, 2.6, -3.4)
  const fill = new HemisphereLight('#fff6ee', '#f0cfb4', 0.6)
  scene.add(key, rim, fill)

  const floor = new Mesh(new PlaneGeometry(9, 9), new ShadowMaterial({ opacity: 0.08 }))
  floor.rotation.x = -Math.PI / 2
  floor.receiveShadow = true
  scene.add(floor, contactShadow())

  const camera = new PerspectiveCamera(VIEW.fov, 1, 0.1, 60)
  const place = () => {
    const [x, y, z] = VIEW.target
    camera.position.set(x, y + Math.sin(VIEW.pitch) * VIEW.distance, z + Math.cos(VIEW.pitch) * VIEW.distance)
    camera.lookAt(x, y, z)
  }
  place()

  const resize = () => {
    const width = Math.max(1, host.clientWidth)
    const height = Math.max(1, host.clientHeight)
    renderer.setSize(width, height, false)
    camera.aspect = width / height
    // Keep the whole figure in view on a tall, narrow window.
    const half = Math.max(Math.tan((VIEW.fov / 2) * (Math.PI / 180)), VIEW.halfWidth / VIEW.distance / camera.aspect)
    camera.fov = 2 * Math.atan(half) * (180 / Math.PI)
    camera.updateProjectionMatrix()
  }
  resize()
  window.addEventListener('resize', resize)

  return {
    scene, camera, renderer, resize,
    render: () => renderer.render(scene, camera),
    dispose() {
      window.removeEventListener('resize', resize)
      renderer.dispose()
      renderer.domElement.remove()
    },
  }
}
