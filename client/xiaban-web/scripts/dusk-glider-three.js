// The three.js subset bundled for the dusk-glider pack. The host iframe has an opaque origin, so the pack
// cannot load ES modules; `npm run build:glider-three` turns this list into a classic script exposing `THREE`.
export {
  WebGLRenderer, Scene, PerspectiveCamera, Group, Object3D, Mesh, InstancedMesh, Points, Line, LineSegments, Sprite,
  BufferGeometry, BufferAttribute, Float32BufferAttribute, InstancedBufferAttribute,
  PlaneGeometry, BoxGeometry, SphereGeometry, IcosahedronGeometry, DodecahedronGeometry, OctahedronGeometry, TetrahedronGeometry,
  ConeGeometry, CylinderGeometry, TorusGeometry, LatheGeometry, CircleGeometry, RingGeometry, CapsuleGeometry,
  MeshLambertMaterial, MeshStandardMaterial, MeshPhongMaterial, MeshBasicMaterial, ShaderMaterial, PointsMaterial, SpriteMaterial, LineBasicMaterial,
  Color, Vector2, Vector3, Vector4, Matrix4, Quaternion, Euler, MathUtils, Fog, Clock,
  DirectionalLight, HemisphereLight, AmbientLight, PointLight,
  CanvasTexture, DataTexture, Texture,
  UniformsLib, UniformsUtils, ShaderChunk,
  AdditiveBlending, NormalBlending, DoubleSide, BackSide, FrontSide,
  ACESFilmicToneMapping, AgXToneMapping, NeutralToneMapping, SRGBColorSpace, PCFSoftShadowMap, PCFShadowMap,
  LinearFilter, RepeatWrapping, ClampToEdgeWrapping, DynamicDrawUsage,
} from 'three'
export { mergeGeometries, mergeVertices } from 'three/addons/utils/BufferGeometryUtils.js'
