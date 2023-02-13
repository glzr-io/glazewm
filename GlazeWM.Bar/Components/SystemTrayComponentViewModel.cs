using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Linq;
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
    public ObservableCollection<NotifyIconViewModel> PinnedTrayIcons = new();
    public ObservableCollection<NotifyIconViewModel> UnpinnedTrayIcons = new();
    public ICollectionView _pinnedNotifyIcons;
    public ICollectionView PinnedNotifyIcons
    {
      get => _pinnedNotifyIcons;
      set
      {
        _pinnedNotifyIcons = value;
        OnPropertyChanged(nameof(PinnedNotifyIcons));
        PinnedTrayIcons = new ObservableCollection<NotifyIconViewModel>(_notificationArea.TrayIcons.Where(e => e.IsPinned).Select(e => new NotifyIconViewModel(e)).ToList());
        OnPropertyChanged(nameof(PinnedTrayIcons));
      }
    }
    public ICollectionView _unpinnedNotifyIcons;
    public ICollectionView UnpinnedNotifyIcons
    {
      get => _unpinnedNotifyIcons;
      set
      {
        _unpinnedNotifyIcons = value;
        OnPropertyChanged(nameof(UnpinnedNotifyIcons));
        UnpinnedTrayIcons = new ObservableCollection<NotifyIconViewModel>(_notificationArea.TrayIcons.Where(e => !e.IsPinned).Select(e => new NotifyIconViewModel(e)).ToList());
        OnPropertyChanged(nameof(UnpinnedTrayIcons));
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
      _shellManager ??= new ShellManager();
      _notificationArea = _shellManager.NotificationArea;
      UnpinnedNotifyIcons = _notificationArea.UnpinnedIcons;
      PinnedNotifyIcons = _notificationArea.PinnedIcons;
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
