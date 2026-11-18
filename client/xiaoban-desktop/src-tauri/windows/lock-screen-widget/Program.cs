using System.Runtime.InteropServices;
using Microsoft.Windows.Widgets.Providers;
using WinRT;

namespace Xiaoban.LockScreenWidget;

internal static class Program
{
    private static readonly Guid ProviderClass = new("4c1ed520-6acf-4cd8-bc9e-0c61ca553cd4");
    private static readonly AutoResetEvent Activity = new(false);

    [MTAThread]
    private static void Main()
    {
        ComWrappersSupport.InitializeComWrappers();
        var factory = new WidgetFactory();
        const int localServer = 0x4;
        const int multipleUse = 0x1;
        Marshal.ThrowExceptionForHR(RegisterClass(ProviderClass, factory, localServer, multipleUse, out uint cookie));
        try
        {
            // The host can activate the package again. Keep no background worker or timer of our own.
            while (Activity.WaitOne(TimeSpan.FromMinutes(2))) { }
        }
        finally
        {
            RevokeClass(cookie);
            GC.KeepAlive(factory);
        }
    }

    internal static void NoteActivity() => Activity.Set();

    [DllImport("ole32.dll", EntryPoint = "CoRegisterClassObject")]
    private static extern int RegisterClass(
        [MarshalAs(UnmanagedType.LPStruct)] Guid clsid,
        [MarshalAs(UnmanagedType.IUnknown)] object factory, int context, int flags, out uint cookie);

    [DllImport("ole32.dll", EntryPoint = "CoRevokeClassObject")]
    private static extern int RevokeClass(uint cookie);
}

[ComImport, InterfaceType(ComInterfaceType.InterfaceIsIUnknown), Guid("00000001-0000-0000-C000-000000000046")]
internal interface IClassFactory
{
    [PreserveSig]
    int CreateInstance(nint outer, ref Guid iid, out nint instance);

    [PreserveSig]
    int LockServer([MarshalAs(UnmanagedType.Bool)] bool locked);
}

[ComVisible(true)]
internal sealed class WidgetFactory : IClassFactory
{
    public int CreateInstance(nint outer, ref Guid iid, out nint instance)
    {
        instance = 0;
        if (outer != 0) return unchecked((int)0x80040110); // CLASS_E_NOAGGREGATION
        Program.NoteActivity();
        nint provider = 0;
        try
        {
            provider = MarshalInspectable<IWidgetProvider>.FromManaged(new WeatherProvider());
            return Marshal.QueryInterface(provider, in iid, out instance);
        }
        catch (Exception error)
        {
            return Marshal.GetHRForException(error);
        }
        finally
        {
            if (provider != 0) Marshal.Release(provider);
        }
    }

    public int LockServer(bool locked)
    {
        if (locked) Program.NoteActivity();
        return 0;
    }
}
