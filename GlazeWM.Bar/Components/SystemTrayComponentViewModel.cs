using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Windows.Input;
using GlazeWM.Bar.Common;
using GlazeWM.Domain.UserConfigs;
using ManagedShell;

namespace GlazeWM.Bar.Components
{
  public class SystemTrayComponentViewModel : ComponentViewModel
  {
    private readonly SystemTrayComponentConfig _config;

    private static ShellManager _shellManager { get; set; }

    public ICommand ToggleShowAllIconsCommand => new RelayCommand(ToggleShowAllIcons);

    private bool IsExpanded { get; set; } = true;

    public string ExpandCollapseText => IsExpanded
      ? _config.LabelExpandText
      : _config.LabelCollapseText;

    public static ObservableCollection<NotifyIconViewModel> PinnedTrayIcons =>
      new(_pinnedTrayIcons);
    public static ObservableCollection<NotifyIconViewModel> UnpinnedTrayIcons =>
      new(_unpinnedTrayIcons);

    private static IEnumerable<NotifyIconViewModel> _pinnedTrayIcons =>
      _shellManager.NotificationArea.PinnedIcons
        .Cast<ManagedShell.WindowsTray.NotifyIcon>()
        .Select(trayIcon => new NotifyIconViewModel(trayIcon));

    private static IEnumerable<NotifyIconViewModel> _unpinnedTrayIcons =>
      _shellManager.NotificationArea.UnpinnedIcons
        .Cast<ManagedShell.WindowsTray.NotifyIcon>()
        .Select(trayIcon => new NotifyIconViewModel(trayIcon));

    public SystemTrayComponentViewModel(
      BarViewModel parentViewModel,
      SystemTrayComponentConfig config) : base(parentViewModel, config)
    {
      _config = config;
      _shellManager ??= new ShellManager();
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
