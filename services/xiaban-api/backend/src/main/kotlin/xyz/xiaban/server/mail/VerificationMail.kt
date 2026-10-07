package xyz.xiaban.server.mail

fun verificationMail(to: String, code: String): Mail {
    require(Regex("^[0-9]{6}$").matches(code))
    val text = "小伴把你的登录暗号送到啦 🐾\n\n验证码：$code\n\n10 分钟内有效，仅可使用一次。用于登录、注册或保存你的账号。\n请勿告诉别人；如果不是你本人操作，忽略这封邮件就好。\n\n— 下班啦"
    val html = """<!doctype html><html lang="zh-CN"><head><meta charset="utf-8"></head>
      <body style="margin:0;background:#f5f2ec;font-family:Arial,'Microsoft YaHei',sans-serif;color:#243149">
      <div style="display:none;max-height:0;overflow:hidden">小伴的登录暗号来了：10 分钟有效，请勿转发。</div>
      <table role="presentation" width="100%" cellpadding="0" cellspacing="0"><tr><td style="padding:32px 16px" align="center">
      <table role="presentation" width="480" cellpadding="0" cellspacing="0" style="width:100%;max-width:480px;border-radius:20px;overflow:hidden;background:#fff">
      <tr><td style="padding:28px 32px;background:#182338;color:#edbd76;font-size:18px;font-weight:bold">🐾 下班啦 · 小伴来送信</td></tr>
      <tr><td style="padding:30px 32px 12px;font-size:24px;font-weight:bold">门口的暗号，给你留好啦。</td></tr>
      <tr><td style="padding:0 32px 22px;font-size:15px;line-height:1.8;color:#647086">小伴把门打开了一条缝。输入下面这串数字，就能回到你的小天地。</td></tr>
      <tr><td style="padding:0 32px"><div style="padding:24px 12px;border:2px dashed #e5b46e;border-radius:14px;background:#fff6e7;text-align:center">
      <div style="font-size:12px;color:#927044;letter-spacing:2px">你的 6 位验证码</div>
      <div style="margin:12px 0;font-family:'Courier New',monospace;font-size:40px;font-weight:bold;letter-spacing:6px;color:#182338">$code</div>
      <div style="font-size:14px;font-weight:bold;color:#927044">10 分钟内有效 · 仅可使用一次</div></div></td></tr>
      <tr><td style="padding:24px 32px 30px;font-size:13px;line-height:1.9;color:#7a8494">暗号只给你一个人，请勿转发或告诉别人。<br>不是你本人操作？忽略这封信就好，小伴不会打扰你。</td></tr>
      <tr><td style="padding:18px 32px;background:#faf8f3;color:#9b8e7d;font-size:12px">今天也辛苦啦，记得给自己留一点休息时间。 ☾</td></tr>
      </table></td></tr></table></body></html>""".trimIndent()
    return Mail(to, "下班啦验证码：$code", text, html)
}
