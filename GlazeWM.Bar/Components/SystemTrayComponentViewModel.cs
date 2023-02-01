using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Linq;
using System.Reactive.Linq;
using System.Windows.Input;
using System.Windows.Threading;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.Containers.Events;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Domain.Workspaces.Commands;
using GlazeWM.Domain.Workspaces.Events;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Bar.Components
{
  public class SystemTrayComponentViewModel : ComponentViewModel
  {
    private readonly SystemTrayComponentConfig _systemTrayComponentConfig;
    public string ExpandCollapseText { get; set; } = string.Empty;
    public ICommand ToggleShowAllIconsCommand => new RelayCommand(ToggleShowAllIcons);
    private bool isExpanded;

    public SystemTrayComponentViewModel(
      BarViewModel parentViewModel,
      SystemTrayComponentConfig config) : base(parentViewModel, config)
    {
      _systemTrayComponentConfig = config;
      ExpandCollapseText = config.CollapseText;
    }

    public void ToggleShowAllIcons()
    {
      Debug.WriteLine("EHERE");
      if (isExpanded)
      {
        // NotifyIconsUnpinned.ItemsSource = null;
        ExpandCollapseText = "";
        // ToggleShowAllIconsBtn.Content = "";
        isExpanded = false;
      }
      else
      {
        // NotifyIconsUnpinned.ItemsSource = notificationArea.UnpinnedIcons;
        ExpandCollapseText = "";
        isExpanded = true;
      }
    }

  }
}
