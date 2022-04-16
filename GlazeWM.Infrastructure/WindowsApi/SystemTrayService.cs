using GlazeWM.Infrastructure.Bussing;
using System;
using System.Drawing;
using System.Windows.Forms;
using System.Threading;
using GlazeWM.Infrastructure.WindowsApi.Events;
using System.Reflection;
using System.IO;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemTrayService
  {
    private readonly Bus _bus;

    public SystemTrayService(Bus bus)
    {
      _bus = bus;
    }

    public void AddToSystemTray()
    {
      var thread = new Thread(() => PopulateSystemTray());
      thread.Name = "GlazeWMSystemTray";
      thread.Start();
    }

    private void PopulateSystemTray()
    {
      var contextMenuStrip = new ContextMenuStrip();

      contextMenuStrip.Items.Add("Exit", null, SignalApplicationExit);

      var assembly = Assembly.GetEntryAssembly();
      const string iconResourceName = "GlazeWM.Bootstrapper.icon.ico";

      // Get the embedded icon resource from the entry assembly.
      using (Stream stream = assembly.GetManifestResourceStream(iconResourceName))
      {
        var notificationIcon = new NotifyIcon()
        {
          Icon = new Icon(stream),
          ContextMenuStrip = contextMenuStrip,
          Text = "GlazeWM"
        };

        notificationIcon.Visible = true;
      }

      // System tray requires a message loop within the thread that is executing the code.
      Application.Run();
    }

    void SignalApplicationExit(object sender, EventArgs e)
    {
      // TODO: Call `Dispose()` on `notificationIcon`.
      _bus.RaiseEvent(new ApplicationExitingEvent());
    }
  }
}
