package xyz.xiaban.wallpaper.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.compose.ui.window.Dialog
import androidx.compose.ui.window.DialogProperties
import xyz.xiaban.wallpaper.data.Announcement

/**
 * A notice from the website: title, text, and the buttons. A skippable notice has cancel and confirm and can be
 * dismissed with the back key or a tap outside; a mandatory one has only confirm, which is the one way out.
 * Confirm opens the notice's link when it has one, and in every case closes the dialog.
 */
@Composable
fun AnnouncementDialog(notice: Announcement, onConfirm: () -> Unit, onCancel: () -> Unit) {
    val dismissable = notice.skippable
    Dialog(
        onDismissRequest = { if (dismissable) onCancel() },
        properties = DialogProperties(dismissOnBackPress = dismissable, dismissOnClickOutside = dismissable, usePlatformDefaultWidth = false),
    ) {
        Column(
            Modifier.padding(horizontal = 28.dp).widthIn(max = 380.dp).fillMaxWidth()
                .clip(RoundedCornerShape(24.dp)).background(Palette.Card)
                .padding(start = 24.dp, end = 24.dp, top = 26.dp, bottom = 22.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
        ) {
            Text(notice.title, color = Palette.Text, fontSize = 18.sp, fontWeight = FontWeight.Bold, textAlign = TextAlign.Center)
            if (notice.body.isNotEmpty()) {
                Spacer(Modifier.height(14.dp))
                // A long notice scrolls inside the card instead of pushing the buttons off the screen.
                Column(Modifier.fillMaxWidth().heightIn(max = 320.dp).verticalScroll(rememberScrollState())) {
                    Text(notice.body, color = Palette.Sub, fontSize = 14.sp, lineHeight = 22.sp)
                }
            }
            Spacer(Modifier.height(24.dp))
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                if (notice.skippable) {
                    DialogButton(notice.cancelText, filled = false, onClick = onCancel, modifier = Modifier.weight(1f))
                }
                DialogButton(notice.confirmText, filled = true, onClick = onConfirm, modifier = Modifier.weight(1f))
            }
        }
    }
}

@Composable
private fun DialogButton(text: String, filled: Boolean, onClick: () -> Unit, modifier: Modifier) {
    Box(
        modifier.height(46.dp).clip(CircleShape).background(if (filled) Palette.Brand else Palette.Field).clickable(onClick = onClick),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text,
            color = if (filled) Color.White else Palette.Text,
            fontSize = 15.sp,
            fontWeight = if (filled) FontWeight.Bold else FontWeight.Medium,
            maxLines = 1,
        )
    }
}

/** What the confirm button does: open the link if there is one. Closing is done by the caller either way. */
@Composable
fun rememberAnnouncementConfirm(app: AppViewModel, notice: Announcement): () -> Unit {
    val context = LocalContext.current
    return {
        app.dismissAnnouncement()
        notice.confirmLink?.let { openLink(context, it) { app.say("没有找到可以打开链接的应用") } }
    }
}
