using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure;
using ManagedShell;

namespace GlazeWM.Bar.Components
{
  public class SystemTrayComponentViewModel : ComponentViewModel
  {
    private readonly SystemTrayComponentConfig _config;

    private readonly ShellManager _shellManager =
      ServiceLocator.GetRequiredService<ShellManager>();

    public ICommand ToggleShowAllIconsCommand => new RelayCommand(ToggleShowAllIcons);

    private bool _isExpanded = true;
    public bool IsExpanded
    {
      get => _isExpanded;
      set
      {
        _isExpanded = value;
        OnPropertyChanged(nameof(IsExpanded));
      }
    }

    public string ExpandCollapseText => IsExpanded
      ? _config.LabelExpandText
      : _config.LabelCollapseText;

    public ObservableCollection<NotifyIconViewModel> PinnedTrayIcons =>
      new(_pinnedTrayIcons);
    public ObservableCollection<NotifyIconViewModel> UnpinnedTrayIcons =>
      new(_unpinnedTrayIcons);

    private IEnumerable<NotifyIconViewModel> _pinnedTrayIcons =>
      _shellManager.NotificationArea.PinnedIcons
        .Cast<ManagedShell.WindowsTray.NotifyIcon>()
        .Select(trayIcon => new NotifyIconViewModel(trayIcon));

    private IEnumerable<NotifyIconViewModel> _unpinnedTrayIcons =>
      _shellManager.NotificationArea.UnpinnedIcons
        .Cast<ManagedShell.WindowsTray.NotifyIcon>()
        .Select(trayIcon => new NotifyIconViewModel(trayIcon));

    public SystemTrayComponentViewModel(
      BarViewModel parentViewModel,
      SystemTrayComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;

      // Subscribe to collection changes of pinned/unpinned tray icons.
      _shellManager.NotificationArea.UnpinnedIcons.CollectionChanged +=
        (_, _) => OnPropertyChanged(nameof(UnpinnedTrayIcons));
      _shellManager.NotificationArea.PinnedIcons.CollectionChanged +=
        (_, _) => OnPropertyChanged(nameof(PinnedTrayIcons));
    }

    public void ToggleShowAllIcons()
    {
      IsExpanded = !IsExpanded;
      OnPropertyChanged(nameof(ExpandCollapseText));
    }
  }
}
