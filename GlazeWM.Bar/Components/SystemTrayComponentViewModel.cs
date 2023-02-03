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
    public string _ExpandCollapseText = string.Empty;
    public string ExpandCollapseText
    {
      get => _ExpandCollapseText;
      set
      {
        _ExpandCollapseText = value;
        OnPropertyChanged(nameof(ExpandCollapseText));
      }
    }
    public ICommand ToggleShowAllIconsCommand => new RelayCommand(ToggleShowAllIcons);
    public ICollectionView PinnedNotifyIcons { get; set; }
    public ICollectionView _unpinnedNotifyIcons;
    public ICollectionView UnpinnedNotifyIcons
    {
      get => _unpinnedNotifyIcons;
      set
      {
        _unpinnedNotifyIcons = value;
        OnPropertyChanged(nameof(UnpinnedNotifyIcons));
      }
    }
    private bool _isExpanded { get; set; } = true;
    private static NotificationArea _notificationArea { get; set; }
    private static ShellManager _shellManager { get; set; }

    public SystemTrayComponentViewModel(
      BarViewModel parentViewModel,
      SystemTrayComponentConfig config) : base(parentViewModel, config)
    {
      _systemTrayComponentConfig = config;

      if (_shellManager == null)
      {
        _shellManager = new ShellManager();
      }

      _notificationArea = _shellManager.NotificationArea;
      UnpinnedNotifyIcons = _notificationArea.UnpinnedIcons;
      PinnedNotifyIcons = _notificationArea.PinnedIcons;
      OnPropertyChanged(nameof(ExpandCollapseText));
      OnPropertyChanged(nameof(UnpinnedNotifyIcons));

      ExpandCollapseText = config.LabelCollapseText;
    }

    public void ToggleShowAllIcons()
    {
      if (_isExpanded)
      {
        UnpinnedNotifyIcons = null;
        ExpandCollapseText = _systemTrayComponentConfig.LabelExpandText;
      }
      else
      {
        UnpinnedNotifyIcons = _notificationArea.UnpinnedIcons;
        ExpandCollapseText = _systemTrayComponentConfig.LabelCollapseText;
      }
      _isExpanded = !_isExpanded;
    }
  }
}
