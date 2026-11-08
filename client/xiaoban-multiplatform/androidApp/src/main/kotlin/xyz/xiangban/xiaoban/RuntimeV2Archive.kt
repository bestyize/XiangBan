package xyz.xiangban.xiaoban

import java.io.ByteArrayOutputStream
import java.io.File
import java.io.OutputStream
import java.io.RandomAccessFile
import java.util.Locale
import java.util.zip.CRC32
import java.util.zip.ZipEntry
import org.apache.commons.compress.archivers.zip.ZipArchiveEntry
import org.apache.commons.compress.archivers.zip.ZipFile

internal object RuntimeV2Archive {
    const val MAX_ZIP_BYTES = 100L * 1024 * 1024
    private const val MAX_EXPANDED_BYTES = 100L * 1024 * 1024
    private const val MAX_FILE_BYTES = 50L * 1024 * 1024
    private const val MAX_MANIFEST_BYTES = 64 * 1024
    private const val MAX_ENTRIES = 3000
    private const val MANIFEST = "manifest.json"

    /** Extracts into a fresh, private staging directory. The caller publishes it only after success. */
    fun extract(zip: File, destination: File, expectedKind: String): RuntimeV2Manifest {
        require(expectedKind == "app" || expectedKind == "game") { "作品类型无效" }
        require(destination.isDirectory && destination.list()?.isEmpty() == true) { "作品暂存目录必须为空" }
        try {
            return RandomAccessFile(zip, "r").use { input ->
                val directory = validateCentralDirectory(input)
                ZipFile.builder().setSeekableByteChannel(input.channel).setMaxNumberOfDisks(1)
                    .setUseUnicodeExtraFields(false).setIgnoreLocalFileHeader(true).get().use { archive ->
                        val entries = validateEntries(archive, input, directory)
                        val manifestEntry = entries[MANIFEST]?.takeUnless { it.isDirectory }
                            ?: throw IllegalArgumentException("压缩包根目录缺少 manifest.json")
                        val buffer = ByteArray(DEFAULT_BUFFER_SIZE)
                        val manifestBytes = ByteArrayOutputStream(manifestEntry.size.toInt()).also {
                            archive.copyChecked(manifestEntry, it, buffer)
                        }.toByteArray()
                        val manifest = RuntimeV2Manifest.read(manifestBytes.decodeToString(throwOnInvalidSequence = true))
                        require(manifest.kind == expectedKind) { "作品清单类型与发布类型不一致" }
                        listOf(manifest.entry, manifest.icon, manifest.cover).forEach { path ->
                            require(entries[path]?.isDirectory == false) { "作品缺少 $path" }
                        }
                        for ((path, entry) in entries) {
                            val target = File(destination, path)
                            if (entry.isDirectory) {
                                archive.copyChecked(entry, null, buffer)
                                ensureDirectory(target)
                            } else {
                                ensureDirectory(checkNotNull(target.parentFile))
                                target.outputStream().buffered().use { output ->
                                    if (path == MANIFEST) output.write(manifestBytes)
                                    else archive.copyChecked(entry, output, buffer)
                                }
                            }
                        }
                        manifest
                    }
            }
        } catch (error: Exception) {
            destination.listFiles()?.forEach { it.deleteRecursively() }
            throw error
        }
    }

    private fun validateEntries(archive: ZipFile, input: RandomAccessFile, directory: Directory): Map<String, ZipArchiveEntry> {
        val entries = linkedMapOf<String, ZipArchiveEntry>()
        val paths = mutableMapOf<String, Pair<String, Boolean>>()
        fun claim(path: String, isDirectory: Boolean) {
            val exact = path to isDirectory
            val previous = paths.putIfAbsent(path.lowercase(Locale.ROOT), exact)
            require(previous == null || previous == exact) { "作品路径存在大小写或文件目录冲突" }
        }

        val source = archive.entriesInPhysicalOrder
        var expanded = 0L
        var previousEnd = 0L
        while (source.hasMoreElements()) {
            require(entries.size < directory.count) { "压缩包目录条目数不一致" }
            val entry = source.nextElement()
            val rawName = entry.rawName
            val name = rawName.toString(Charsets.US_ASCII)
            val path = if (entry.isDirectory) name.removeSuffix("/") else name
            require(name == entry.name && isPackagePath(path)) { "压缩包包含不安全的路径" }
            require(entry.isDirectory || packageMimeType(path) != null) { "压缩包包含不支持的文件类型" }
            val fileMode = (entry.externalAttributes ushr 16).toInt() and 0xf000
            val expectedMode = if (entry.isDirectory) 0x4000 else 0x8000
            require(fileMode == 0 || fileMode == expectedMode) { "压缩包包含链接或特殊文件" }
            require(entry.isDirectory || (entry.externalAttributes and 0x10L) == 0L) { "压缩包的文件与目录标记不一致" }
            require(entry.method == ZipEntry.STORED || entry.method == ZipEntry.DEFLATED) { "压缩方式不受支持" }
            require(entry.diskNumberStart == 0L && archive.canReadEntryData(entry)) { "压缩包不能包含分卷或加密文件" }
            val maxSize = when {
                entry.isDirectory -> 0L
                path == MANIFEST -> MAX_MANIFEST_BYTES.toLong()
                else -> MAX_FILE_BYTES
            }
            require(entry.size in 0..maxSize && entry.crc in 0..0xffffffffL) { "作品文件大小或校验信息无效" }
            expanded += entry.size
            require(expanded <= MAX_EXPANDED_BYTES) { "作品解压后超过 100 MB" }
            previousEnd = validateLocalHeader(input, entry, rawName, previousEnd, directory.offset)
            path.indices.filter { path[it] == '/' }.forEach { claim(path.substring(0, it), true) }
            claim(path, entry.isDirectory)
            require(entries.put(path, entry) == null) { "压缩包包含重复条目" }
        }
        require(entries.size == directory.count) { "压缩包目录条目数不一致" }
        return entries
    }

    private fun validateLocalHeader(input: RandomAccessFile, entry: ZipArchiveEntry, rawName: ByteArray,
        previousEnd: Long, directoryOffset: Long): Long {
        val localOffset = entry.localHeaderOffset
        require(localOffset in previousEnd..directoryOffset - 30) { "压缩包文件数据越界" }
        input.seek(localOffset)
        require(input.u32() == 0x04034b50L) { "压缩包文件头损坏" }
        input.skipBytes(2)
        require(input.u16() == entry.rawFlag && input.u16() == entry.method) { "压缩包文件头标记不一致" }
        input.skipBytes(16)
        val nameLength = input.u16()
        val extraLength = input.u16()
        val dataOffset = localOffset + 30 + nameLength + extraLength
        require(nameLength == rawName.size && dataOffset <= directoryOffset && entry.compressedSize in 0..directoryOffset - dataOffset) {
            "压缩包文件头长度无效"
        }
        val name = ByteArray(nameLength).also(input::readFully)
        require(name.contentEquals(rawName)) { "压缩包文件头路径不一致" }
        return dataOffset + entry.compressedSize
    }

    private fun ZipFile.copyChecked(entry: ZipArchiveEntry, output: OutputStream?, buffer: ByteArray) {
        val crc = CRC32()
        var size = 0L
        requireNotNull(getInputStream(entry)) { "压缩包文件不可读取" }.use { stream ->
            while (true) {
                val read = stream.read(buffer)
                if (read == -1) break
                size += read
                require(size <= entry.size) { "压缩包文件解压后超过声明大小" }
                crc.update(buffer, 0, read)
                output?.write(buffer, 0, read)
            }
        }
        require(size == entry.size && crc.value == entry.crc) { "压缩包文件校验失败" }
    }

    private fun ensureDirectory(directory: File) {
        check(directory.isDirectory || directory.mkdirs()) { "无法创建作品目录" }
    }

    private data class Directory(val offset: Long, val count: Int)

    // Commons Compress indexes the whole central directory when opening. Bound its actual entry count first.
    private fun validateCentralDirectory(input: RandomAccessFile): Directory {
        val length = input.length()
        require(length in 22..MAX_ZIP_BYTES) { "作品压缩包大小无效" }
        val tail = ByteArray(length.coerceAtMost(22L + 65535).toInt())
        input.seek(length - tail.size)
        input.readFully(tail)
        val endIndex = (tail.size - 22 downTo 0).firstOrNull { index ->
            tail[index] == 0x50.toByte() && tail[index + 1] == 0x4b.toByte() &&
                tail[index + 2] == 0x05.toByte() && tail[index + 3] == 0x06.toByte()
        } ?: throw IllegalArgumentException("缺少 ZIP 目录尾记录")
        val footer = length - tail.size + endIndex
        input.seek(footer + 4)
        require(input.u16() == 0 && input.u16() == 0) { "作品不支持分卷压缩包" }
        val diskCount = input.u16().toLong()
        val totalCount = input.u16().toLong()
        val size = input.u32()
        val offset = input.u32()
        require(diskCount == totalCount && footer + 22 + input.u16() == length) { "ZIP 目录尾记录无效" }

        val zip64 = readZip64Directory(input, footer)
        val count = zip64?.count ?: totalCount
        val directorySize = zip64?.size ?: size
        val directoryOffset = zip64?.offset ?: offset
        val directoryEnd = zip64?.footer ?: footer
        require(count in 1..MAX_ENTRIES.toLong()) { "作品压缩包条目数无效" }
        require((totalCount == 65535L || count == totalCount) && (size == 0xffffffffL || directorySize == size) &&
            (offset == 0xffffffffL || directoryOffset == offset)) { "ZIP 目录尾记录不一致" }
        require(directoryOffset in 0..directoryEnd && directorySize == directoryEnd - directoryOffset) { "ZIP 目录范围无效" }

        input.seek(directoryOffset)
        repeat(count.toInt()) {
            require(input.filePointer + 46 <= directoryEnd && input.u32() == 0x02014b50L) { "ZIP 目录条目损坏" }
            input.skipBytes(24)
            val variableLength = input.u16() + input.u16() + input.u16()
            val next = input.filePointer + 12 + variableLength
            require(next <= directoryEnd) { "ZIP 目录条目越界" }
            input.seek(next)
        }
        require(input.filePointer == directoryEnd) { "ZIP 目录条目数不一致" }
        return Directory(directoryOffset, count.toInt())
    }

    private data class Zip64Directory(val count: Long, val size: Long, val offset: Long, val footer: Long)

    private fun readZip64Directory(input: RandomAccessFile, footer: Long): Zip64Directory? {
        val locator = footer - 20
        if (locator < 0) return null
        input.seek(locator)
        if (input.u32() != 0x07064b50L) return null
        require(input.u32() == 0L) { "作品不支持分卷压缩包" }
        val zip64Footer = input.i64()
        require(input.u32() == 1L && zip64Footer in 0..locator - 56) { "ZIP64 定位记录无效" }
        input.seek(zip64Footer)
        require(input.u32() == 0x06064b50L && input.i64() == locator - zip64Footer - 12) { "ZIP64 目录尾记录无效" }
        input.skipBytes(4)
        require(input.u32() == 0L && input.u32() == 0L) { "作品不支持分卷压缩包" }
        val count = input.i64()
        require(input.i64() == count) { "ZIP64 目录条目数无效" }
        return Zip64Directory(count, input.i64(), input.i64(), zip64Footer)
    }

    private fun RandomAccessFile.u16(): Int = java.lang.Short.reverseBytes(readShort()).toInt() and 0xffff
    private fun RandomAccessFile.u32(): Long = Integer.reverseBytes(readInt()).toLong() and 0xffffffffL
    private fun RandomAccessFile.i64(): Long = java.lang.Long.reverseBytes(readLong())
}
