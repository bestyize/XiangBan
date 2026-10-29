package coil3
class ComponentRegistryBuilder { fun add(x: Any) {} }
class ImageLoader { class Builder(c: Any?) { fun components(f: ComponentRegistryBuilder.() -> Unit): Builder = this; fun build(): ImageLoader = ImageLoader() } }
