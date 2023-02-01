using System.ComponentModel;
using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;
using ManagedShell;
using ManagedShell.WindowsTray;

namespace GlazeWM.Bar.Components
{
  public class SystemTrayComponentViewModel : ComponentViewModel
  {
    private readonly SystemTrayComponentConfig _systemTrayComponentConfig;
    public string ExpandCollapseText { get; set; } = string.Empty;
    public ICommand ToggleShowAllIconsCommand => new RelayCommand(ToggleShowAllIcons);
    private bool isExpanded = true;
    private static NotificationArea notificationArea;
    private static ShellManager shellManager;
    public ICollectionView PinnedNotifyIcons { get; set; }
    public ICollectionView UnpinnedNotifyIcons { get; set; }

    public SystemTrayComponentViewModel(
      BarViewModel parentViewModel,
      SystemTrayComponentConfig config) : base(parentViewModel, config)
    {
      _systemTrayComponentConfig = config;

      if (shellManager == null)
      {
        // Initialize the default configuration.
        var c = ShellManager.DefaultShellConfig;
        c.EnableTrayService = true;

        // Initialize the shell manager.
        shellManager = new ShellManager(c);
      }

      notificationArea = shellManager.NotificationArea;
      UnpinnedNotifyIcons = notificationArea.UnpinnedIcons;
      PinnedNotifyIcons = notificationArea.PinnedIcons;
      OnPropertyChanged(nameof(ExpandCollapseText));
      OnPropertyChanged(nameof(UnpinnedNotifyIcons));

      ExpandCollapseText = config.CollapseText;
    }

    public void ToggleShowAllIcons()
    {
      if (isExpanded)
      {
        UnpinnedNotifyIcons = null;
        ExpandCollapseText = _systemTrayComponentConfig.ExpandText;
      }
      else
      {
        UnpinnedNotifyIcons = notificationArea.UnpinnedIcons;
        ExpandCollapseText = _systemTrayComponentConfig.CollapseText;
      }
      OnPropertyChanged(nameof(ExpandCollapseText));
      OnPropertyChanged(nameof(UnpinnedNotifyIcons));
      isExpanded = !isExpanded;
    }
  }
}
