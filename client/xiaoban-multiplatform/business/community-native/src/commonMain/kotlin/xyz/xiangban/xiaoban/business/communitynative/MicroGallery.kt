package xyz.xiangban.xiaoban.business.communitynative

internal enum class MicroGalleryExhibit(
    val id: String,
    val title: String,
    val description: String,
    private val cover: String,
    private val page: String,
) {
    Aquarium("aquarium", "水族箱", "看龙鱼和孔雀鱼在山水客厅共游，切换鱼类、背景和灯光。", "living-room", "aquarium.html?scene=living-room"),
    Arowana("arowana", "龙鱼", "近看鳞片反光与鱼身转向。", "arowana", "fish/arowana/preview.html"),
    BlueGuppy("blue-guppy", "蓝扇尾孔雀鱼", "观察尾鳍如何随水摆动。", "guppy", "fish/blue-guppy/preview.html"),
    GuppySchool("guppy-school", "孔雀鱼群", "六条孔雀鱼，四种花色。", "school", "fish/blue-guppy/school-preview.html"),
    PlushRabbit("plush-rabbit", "碎花垂耳兔", "转动 3D 模型，近看绒毛、碎花和蝴蝶结。", "rabbit", "objects/plush-rabbit/preview.html");

    fun coverUrl(forumBase: String) = "${forumBase.trimEnd('/')}/gallery-exhibits/assets/gallery/$cover.webp"

    fun previewUrl(forumBase: String) = "${forumBase.trimEnd('/')}/gallery-exhibits/$page"

    companion object {
        fun byId(id: String): MicroGalleryExhibit? = entries.firstOrNull { it.id == id }
    }
}
