using System;
using System.Drawing;
using System.Reflection;
using System.Windows.Forms;

namespace GlazeWM.Infrastructure.WindowsApi
{
  public class SystemTrayIcon
  {
    private readonly SystemTrayIconConfig _config;
    private NotifyIcon _notifyIcon { get; set; }

    public SystemTrayIcon(SystemTrayIconConfig config)
    {
      _config = config;
    }

    public void Show()
    {
      var contextMenuStrip = new ContextMenuStrip();

      foreach (var action in _config.Actions)
      {
        // Create an entry in the context menu that is shown on click.
        contextMenuStrip.Items.Add(
          action.Key,
          null,
          (object sender, EventArgs e) => action.Value()
        );
      }

      // Get the embedded icon resource from the entry assembly.
      var assembly = Assembly.GetEntryAssembly();
      using var stream = assembly.GetManifestResourceStream(_config.IconResourceName);

      _notifyIcon = new NotifyIcon
      {
        Icon = new Icon(stream),
        ContextMenuStrip = contextMenuStrip,
        Text = _config.HoverText,
        Visible = true
      };
    }

    public void Remove()
    {
      if (_notifyIcon is null)
        return;

      _notifyIcon.Visible = false;
      _notifyIcon.Dispose();
    }
  }
}
