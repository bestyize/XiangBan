package xyz.xiangban.xiaoban

import android.Manifest
import android.app.AppOpsManager
import android.content.Context
import android.content.pm.PackageManager
import android.os.Build
import android.webkit.PermissionRequest
import androidx.activity.ComponentActivity
import androidx.activity.result.contract.ActivityResultContracts
import androidx.core.content.ContextCompat

internal class RuntimeV2Media(
    private val activity: ComponentActivity,
    private val chrome: () -> RuntimeV2Chrome,
    private val visible: () -> Boolean,
    private val onConsentChanged: () -> Unit,
) {
    private enum class Capture(val resource: String, val capability: String, val permission: String, val feature: String, val appOp: String) {
        Camera(PermissionRequest.RESOURCE_VIDEO_CAPTURE, "camera", Manifest.permission.CAMERA, PackageManager.FEATURE_CAMERA_ANY, AppOpsManager.OPSTR_CAMERA),
        Microphone(PermissionRequest.RESOURCE_AUDIO_CAPTURE, "microphone", Manifest.permission.RECORD_AUDIO, PackageManager.FEATURE_MICROPHONE, AppOpsManager.OPSTR_RECORD_AUDIO),
    }

    private data class Pending(val request: PermissionRequest, val host: String, val captures: Set<Capture>)
    private val preferences by lazy { activity.getSharedPreferences("runtime_v2_media", Context.MODE_PRIVATE) }
    private var pending: Pending? = null
    private var systemPromptOpen = false
    private val granted = mutableSetOf<Capture>()
    private val active = mutableSetOf<Capture>()
    private var stopWatching: (() -> Unit)? = null
    private val requestPermissions = activity.registerForActivityResult(ActivityResultContracts.RequestMultiplePermissions()) {
        systemPromptOpen = false
        pending?.let(::complete)
    }

    fun capabilities(pack: RuntimeV2Package): List<String> = Capture.entries.filter {
        it.capability in pack.manifest.capabilities && activity.packageManager.hasSystemFeature(it.feature)
    }.map { it.capability }

    fun request(request: PermissionRequest, pack: RuntimeV2Package) {
        val captures = request.resources.map { resource -> Capture.entries.find { it.resource == resource } }
        if (pending != null || systemPromptOpen || !visible() || !pack.ownsOrigin(request.origin) ||
            captures.isEmpty() || captures.any { it == null || it.capability !in capabilities(pack) }
        ) {
            request.deny()
            return
        }
        val item = Pending(request, pack.host, captures.filterNotNull().toSet())
        pending = item
        val needConsent = item.captures.filterNot { preferences.getBoolean(consentKey(item.host, it), false) }
        if (needConsent.isEmpty()) askSystem(item)
        else {
            val (device, data) = when (needConsent.toSet()) {
                setOf(Capture.Camera) -> "摄像头" to "拍到的内容"
                setOf(Capture.Microphone) -> "麦克风" to "录到的声音"
                else -> "摄像头和麦克风" to "拍到的内容和录到的声音"
            }
            chrome().showMediaConsent("《${pack.manifest.name}》想使用${device}。作品可以把${data}发送到网络。") { allowed ->
                onConsentChanged()
                if (pending === item) {
                    if (allowed) askSystem(item) else cancel()
                }
            }
            onConsentChanged()
        }
    }

    fun cancel(request: PermissionRequest? = null) {
        val item = pending ?: return
        if (request != null && request !== item.request) return
        pending = null
        chrome().dismissMediaConsent()
        onConsentChanged()
        if (request == null) item.request.deny()
    }

    private fun askSystem(item: Pending) {
        val missing = item.captures.filterNot(::systemGranted).map { it.permission }.toTypedArray()
        if (missing.isEmpty()) complete(item)
        else {
            systemPromptOpen = true
            requestPermissions.launch(missing)
        }
    }

    private fun complete(item: Pending) {
        pending = null
        if (!item.captures.all(::systemGranted)) {
            item.request.deny()
            return
        }
        val edit = preferences.edit()
        item.captures.forEach { edit.putBoolean(consentKey(item.host, it), true) }
        edit.apply()
        granted.addAll(item.captures)
        item.request.grant(item.captures.map { it.resource }.toTypedArray())
    }

    private fun systemGranted(capture: Capture): Boolean =
        ContextCompat.checkSelfPermission(activity, capture.permission) == PackageManager.PERMISSION_GRANTED

    private fun consentKey(host: String, capture: Capture) = "$host.${capture.capability}"

    fun watch() {
        if (Build.VERSION.SDK_INT < 30 || stopWatching != null) return
        val manager = activity.getSystemService(AppOpsManager::class.java)
        val listener = AppOpsManager.OnOpActiveChangedListener { op, uid, name, running ->
            val capture = Capture.entries.find { it.appOp == op }
            if (capture != null && uid == activity.applicationInfo.uid && name == activity.packageName) {
                if (running && capture in granted) active.add(capture) else active.remove(capture)
                chrome().setMediaActive(active.isNotEmpty())
            }
        }
        manager.startWatchingActive(Capture.entries.map { it.appOp }.toTypedArray(), ContextCompat.getMainExecutor(activity), listener)
        stopWatching = { manager.stopWatchingActive(listener) }
    }

    fun reset() {
        cancel()
        granted.clear()
        active.clear()
        chrome().setMediaActive(false)
    }

    fun close() {
        stopWatching?.invoke()
        stopWatching = null
    }
}
