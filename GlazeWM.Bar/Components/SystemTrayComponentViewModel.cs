using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Data;
using System.Windows.Input;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
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
      if (shellManager == null)
      {
        // Initialize the default configuration.
        var c = ShellManager.DefaultShellConfig;
        c.EnableTrayService = true;

        // Initialize the shell manager.
        shellManager = new ShellManager(c);

        notificationArea = shellManager.NotificationArea;

        UnpinnedNotifyIcons = notificationArea.UnpinnedIcons;
        PinnedNotifyIcons = notificationArea.PinnedIcons;
      }
      _systemTrayComponentConfig = config;
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
