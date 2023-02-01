using System.Windows.Controls;
using System.Windows;
using ManagedShell.WindowsTray;
using ManagedShell;

namespace GlazeWM.Bar.Components
{
  /// <summary>
  /// Interaction logic for SystemTrayComponent.xaml
  /// </summary>
  public partial class SystemTrayComponent : UserControl
  {
    private bool _isLoaded;
    private bool isExpanded = true;
    private static NotificationArea notificationArea;
    private static ShellManager shellManager;

    public SystemTrayComponent()
    {
      if (shellManager == null)
      {
        // Initialize the default configuration.
        var c = ShellManager.DefaultShellConfig;
        c.EnableTrayService = true;
        c.AutoStartTrayService = false;

        // Initialize the shell manager.
        shellManager = new ShellManager(c);
        // Initialize the tray service, since we disabled auto-start above.
        shellManager.NotificationArea.Initialize();
        _ = shellManager.NotificationArea.TrayIcons;
        notificationArea = shellManager.NotificationArea;
      }
      InitializeComponent();
    }

    private void SystemTrayComponent_OnLoaded(object sender, RoutedEventArgs e)
    {
      if (!_isLoaded && notificationArea != null)
      {
        NotifyIconsPinned.ItemsSource = notificationArea.PinnedIcons;
        NotifyIconsUnpinned.ItemsSource = notificationArea.UnpinnedIcons;
        _isLoaded = true;
      }
    }

    private void ToggleShowAllIcons_OnClick(object sender, RoutedEventArgs e)
    {
      if (isExpanded)
      {
        NotifyIconsUnpinned.ItemsSource = null;
        // ToggleShowAllIconsBtn.Content = "";
        // ToggleShowAllIconsBtn.Content = "";
        isExpanded = false;
      }
      else
      {
        NotifyIconsUnpinned.ItemsSource = notificationArea.UnpinnedIcons;
        // ToggleShowAllIconsBtn.Content = "";
        isExpanded = true;
      }
    }
  }
}
