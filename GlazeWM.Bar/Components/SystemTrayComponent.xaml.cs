using System.Windows.Controls;
using System.Collections.ObjectModel;
using System.Collections.Specialized;
using System.ComponentModel;
using System.Windows;
using System.Windows.Data;
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
    private readonly ObservableCollection<NotifyIcon> promotedIcons = new();
    private bool isExpanded = true;
    private static NotificationArea notificationArea;
    private static ShellManager shellManager;

    public SystemTrayComponent()
    {
      if (shellManager == null)
      {
        // Initialize the default configuration.
        var c = ShellManager.DefaultShellConfig;

        // Customize tray service options.
        c.EnableTrayService = true; // controls whether the tray objects are instantiated in ShellManager, true by default
        c.AutoStartTrayService = false; // controls whether the tray service is started as part of ShellManager instantiation, true by default

        // Initialize the shell manager.
        shellManager = new ShellManager(c);
        // Initialize the tray service, since we disabled auto-start above.
        shellManager.NotificationArea.Initialize();
        _ = shellManager.NotificationArea.TrayIcons;
        notificationArea = shellManager.NotificationArea;
      }

      InitializeComponent();
      ToggleShowAllIconsBtn.Content = "";

    }

    private void Settings_PropertyChanged(object sender, PropertyChangedEventArgs e)
    {
      // if (e.PropertyName == "CollapseNotifyIcons")
      // {
      //   if (Settings.Instance.CollapseNotifyIcons)
      //   {
      //     NotifyIcons.ItemsSource = pinnedNotifyIconsSource.View;
      //     SetToggleVisibility();
      //   }
      //   else
      //   {
      // NotifyIconToggleButton.IsChecked = false;
      // NotifyIconToggleButton.Visibility = Visibility.Collapsed;

      // NotifyIcons.ItemsSource = allNotifyIconsSource.View;
      // NotifyIcons.ItemsSource = notificationArea.AllIcons;

      // }
      // }
    }

    private void NotifyIconList_OnLoaded(object sender, RoutedEventArgs e)
    {
      if (!_isLoaded && notificationArea != null)
      {
        CompositeCollection allNotifyIcons = new CompositeCollection();
        allNotifyIcons.Add(new CollectionContainer { Collection = notificationArea.UnpinnedIcons });
        allNotifyIcons.Add(new CollectionContainer { Collection = notificationArea.PinnedIcons });
        // allNotifyIconsSource = new CollectionViewSource { Source = allNotifyIcons };

        CompositeCollection pinnedNotifyIcons = new CompositeCollection();
        pinnedNotifyIcons.Add(new CollectionContainer { Collection = promotedIcons });
        pinnedNotifyIcons.Add(new CollectionContainer { Collection = notificationArea.PinnedIcons });
        // pinnedNotifyIconsSource = new CollectionViewSource { Source = pinnedNotifyIcons };

        notificationArea.UnpinnedIcons.CollectionChanged += UnpinnedIcons_CollectionChanged;

        // NotifyIcons.ItemsSource = allNotifyIconsSource.View;
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
        ToggleShowAllIconsBtn.Content = "";
        isExpanded = false;
      }
      else
      {
        NotifyIconsUnpinned.ItemsSource = notificationArea.UnpinnedIcons;
        ToggleShowAllIconsBtn.Content = "";
        isExpanded = true;
      }
    }

    private void NotifyIconList_OnUnloaded(object sender, RoutedEventArgs e)
    {
      if (!_isLoaded)
      {
        return;
      }

      // Settings.Instance.PropertyChanged -= Settings_PropertyChanged;

      if (notificationArea != null)
      {
        notificationArea.UnpinnedIcons.CollectionChanged -= UnpinnedIcons_CollectionChanged;
      }

      _isLoaded = false;
    }

    private void UnpinnedIcons_CollectionChanged(object sender, NotifyCollectionChangedEventArgs e)
    {
      // SetToggleVisibility();
    }

  }
}