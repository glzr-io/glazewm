using GlazeWM.Infrastructure.Bussing;
using System;
using System.Drawing;
using System.Windows.Forms;
using System.Threading;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemTrayService
  {
    private Bus _bus;

    public SystemTrayService(Bus bus)
    {
      _bus = bus;
    }

    public void AddToSystemTray(string iconPath)
    {
      var thread = new Thread(() => PopulateSystemTray(iconPath));
      thread.Name = "GlazeWMSystemTray";
      thread.Start();
    }

    private void PopulateSystemTray(string iconPath)
    {
      var contextMenuStrip = new ContextMenuStrip();

      ToolStripItem exitMenuItem = new ToolStripMenuItem("Exit");
      contextMenuStrip.Items.Add(exitMenuItem);

      var notificationIcon = new NotifyIcon()
      {
        Icon = new Icon(iconPath),
        ContextMenuStrip = contextMenuStrip,
        Text = "GlazeWM"
      };

      notificationIcon.Visible = true;

      // System tray requires a message loop within the thread that is executing the code.
      Application.Run();
    }

    static void OnApplicationExit(object sender, EventArgs e)
    {
      // TODO: Call `Dispose()` on `notificationIcon`.
      // TODO: Emit bus event that application is exiting.
      Application.Exit();
    }
  }
}
