using System;
using System.Drawing;
using System.Reflection;
using System.Threading;
using System.Windows.Forms;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.WindowsApi.Events;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemTrayService
  {
    private readonly Bus _bus;
    private NotifyIcon _notifyIcon { get; set; }

    public SystemTrayService(Bus bus)
    {
      _bus = bus;
    }

    public void AddToSystemTray()
    {
      var thread = new Thread(() => PopulateSystemTray())
      {
        Name = "GlazeWMSystemTray"
      };
      thread.Start();
    }

    private void PopulateSystemTray()
    {
      var contextMenuStrip = new ContextMenuStrip();

      contextMenuStrip.Items.Add("Exit", null, SignalApplicationExit);

      var assembly = Assembly.GetEntryAssembly();
      const string iconResourceName = "GlazeWM.Bootstrapper.icon.ico";

      // Get the embedded icon resource from the entry assembly.
      using (var stream = assembly.GetManifestResourceStream(iconResourceName))
      {
        _notifyIcon = new NotifyIcon
        {
          Icon = new Icon(stream),
          ContextMenuStrip = contextMenuStrip,
          Text = "GlazeWM",
          Visible = true
        };
      }

      // System tray requires a message loop within the thread that is executing the code.
      Application.Run();
    }

    private void SignalApplicationExit(object sender, EventArgs e)
    {
      _bus.RaiseEvent(new ApplicationExitingEvent());
    }

    public void RemoveFromSystemTray()
    {
      _notifyIcon?.Dispose();
    }
  }
}
