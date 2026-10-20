$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -ReferencedAssemblies System.Windows.Forms.dll,System.Drawing.dll -TypeDefinition @'
using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;
using System.Windows.Forms;
namespace Xiaoban {
  public sealed class RegionCapture : Form {
    [DllImport("user32.dll")] static extern bool SetProcessDPIAware();
    readonly Bitmap desktop;
    Rectangle region;
    Point anchor;
    bool selecting;
    readonly Panel actions = new Panel();
    public RegionCapture() {
      SetProcessDPIAware();
      var screen = SystemInformation.VirtualScreen;
      desktop = new Bitmap(screen.Width, screen.Height, PixelFormat.Format24bppRgb);
      using (var g = Graphics.FromImage(desktop)) { g.CopyFromScreen(screen.Left, screen.Top, 0, 0, screen.Size); }
      AutoScaleMode = AutoScaleMode.None; StartPosition = FormStartPosition.Manual;
      FormBorderStyle = FormBorderStyle.None; Bounds = screen;
      TopMost = true; ShowInTaskbar = false; KeyPreview = true; DoubleBuffered = true;
      Cursor = Cursors.Cross; Text = "小伴截图";
      actions.Size = new Size(250, 42); actions.BackColor = Color.FromArgb(250, 250, 248); actions.Visible = false;
      string[] labels = { "重新截取", "取消", "完成" };
      for (int i=0; i<3; i++) {
        var button = new Button { Text=labels[i], Location=new Point(5+i*80,5), Size=new Size(78,32), FlatStyle=FlatStyle.Flat, Cursor=Cursors.Default, BackColor=Color.FromArgb(250,250,248) };
        button.FlatAppearance.BorderSize=0;
        if(i==0) button.Click += delegate { region=Rectangle.Empty; actions.Hide(); Invalidate(); };
        if(i==1) button.Click += delegate { DialogResult=DialogResult.Cancel; Close(); };
        if(i==2) button.Click += delegate { Confirm(); };
        actions.Controls.Add(button);
      }
      Controls.Add(actions);
    }
    protected override void OnShown(EventArgs e) { base.OnShown(e); Activate(); Focus(); }
    protected override void OnKeyDown(KeyEventArgs e) {
      if(e.KeyCode==Keys.Escape) { DialogResult=DialogResult.Cancel; Close(); e.Handled=true; }
      if(e.KeyCode==Keys.Enter) { Confirm(); e.Handled=true; }
      base.OnKeyDown(e);
    }
    Point Clamped(Point p) { return new Point(Math.Max(0,Math.Min(ClientSize.Width,p.X)),Math.Max(0,Math.Min(ClientSize.Height,p.Y))); }
    protected override void OnMouseDown(MouseEventArgs e) {
      if(e.Button!=MouseButtons.Left) return;
      anchor=Clamped(e.Location); region=Rectangle.Empty; selecting=true; actions.Hide(); Capture=true; Invalidate();
    }
    protected override void OnMouseMove(MouseEventArgs e) {
      if(!selecting) return;
      Point p=Clamped(e.Location);
      region=Rectangle.FromLTRB(Math.Min(anchor.X,p.X),Math.Min(anchor.Y,p.Y),Math.Max(anchor.X,p.X),Math.Max(anchor.Y,p.Y)); Invalidate();
    }
    protected override void OnMouseUp(MouseEventArgs e) {
      if(!selecting) return; selecting=false; Capture=false;
      if(region.Width<12 || region.Height<12) { region=Rectangle.Empty; Invalidate(); return; }
      actions.Location=new Point(Math.Max(0,Math.Min(ClientSize.Width-actions.Width,region.Right-actions.Width)), Math.Max(0,Math.Min(ClientSize.Height-actions.Height,region.Bottom+10)));
      actions.Show(); Invalidate();
    }
    protected override void OnPaint(PaintEventArgs e) {
      e.Graphics.DrawImageUnscaled(desktop,0,0);
      using(var dim=new SolidBrush(Color.FromArgb(95,0,0,0))) {
        if(region.IsEmpty) e.Graphics.FillRectangle(dim,ClientRectangle);
        else {
          e.Graphics.FillRectangle(dim,0,0,ClientSize.Width,region.Top);
          e.Graphics.FillRectangle(dim,0,region.Bottom,ClientSize.Width,ClientSize.Height-region.Bottom);
          e.Graphics.FillRectangle(dim,0,region.Top,region.Left,region.Height);
          e.Graphics.FillRectangle(dim,region.Right,region.Top,ClientSize.Width-region.Right,region.Height);
          using(var pen=new Pen(Color.White,1)) e.Graphics.DrawRectangle(pen,region);
        }
      }
      if(region.IsEmpty) {
        using(var font=new Font("Microsoft YaHei UI",11)) using(var brush=new SolidBrush(Color.White))
          e.Graphics.DrawString("拖动框选截图区域 · Esc 取消",font,brush,24,24);
      }
    }
    void Confirm() { if(region.Width>=12 && region.Height>=12) { DialogResult=DialogResult.OK; Close(); } }
    string Result() {
      double scale=Math.Min(1.0,2048.0/Math.Max(region.Width,region.Height));
      using(var image=new Bitmap(Math.Max(1,(int)(region.Width*scale)),Math.Max(1,(int)(region.Height*scale)),PixelFormat.Format24bppRgb)) {
        using(var g=Graphics.FromImage(image)) { g.InterpolationMode=InterpolationMode.HighQualityBicubic; g.DrawImage(desktop,new Rectangle(Point.Empty,image.Size),region,GraphicsUnit.Pixel); }
        var codec=Array.Find(ImageCodecInfo.GetImageEncoders(), c=>c.MimeType=="image/jpeg");
        using(var parameters=new EncoderParameters(1)) using(var stream=new MemoryStream()) {
          parameters.Param[0]=new EncoderParameter(System.Drawing.Imaging.Encoder.Quality,80L); image.Save(stream,codec,parameters);
          if(stream.Length>3*1024*1024) throw new InvalidOperationException("Screenshot is too large");
          return "data:image/jpeg;base64,"+Convert.ToBase64String(stream.ToArray());
        }
      }
    }
    protected override void Dispose(bool disposing) { if(disposing) desktop.Dispose(); base.Dispose(disposing); }
    public static string Run() {
      using(var form=new RegionCapture()) { return form.ShowDialog()==DialogResult.OK ? form.Result() : ""; }
    }
  }
}
'@
[Xiaoban.RegionCapture]::Run()
