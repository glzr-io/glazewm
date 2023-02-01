using System.Windows.Controls;
using System.Collections.ObjectModel;
using System.Collections.Specialized;
using System.ComponentModel;
using System.Windows;
using System.Windows.Data;
using ManagedShell.WindowsTray;
using ManagedShell;
using System.Diagnostics;
using System.Windows.Input;

namespace GlazeWM.Bar.Components
{
  /// <summary>
  /// Interaction logic for SystemTrayComponent.xaml
  /// </summary>
  public partial class SystemTrayComponent : UserControl
  {
    private bool _isLoaded;
    private CollectionViewSource allNotifyIconsSource;
    private CollectionViewSource pinnedNotifyIconsSource;
    private readonly ObservableCollection<NotifyIcon> promotedIcons = new();

    private bool isExpanded = true;

    public NotificationArea notificationArea;

    public SystemTrayComponent()
    {
      // Initialize the default configuration.
      var c = ShellManager.DefaultShellConfig;

      // Customize tray service options.
      c.EnableTrayService = true; // controls whether the tray objects are instantiated in ShellManager, true by default
      c.AutoStartTrayService = false; // controls whether the tray service is started as part of ShellManager instantiation, true by default
      c.PinnedNotifyIcons = new[] { "GUID or PathToExe:UID" }; // sets the initial NotifyIcons that should be included in the PinnedIcons collection, by default Action Center (prior to Windows 10 only), Power, Network, and Volume.

      // Initialize the shell manager.
      var _shellManager = new ShellManager(c);

      // Initialize the tray service, since we disabled auto-start above.
      _shellManager.NotificationArea.Initialize();
      _ = _shellManager.NotificationArea.TrayIcons;
      notificationArea = _shellManager.NotificationArea;


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
        allNotifyIconsSource = new CollectionViewSource { Source = allNotifyIcons };

        CompositeCollection pinnedNotifyIcons = new CompositeCollection();
        pinnedNotifyIcons.Add(new CollectionContainer { Collection = promotedIcons });
        pinnedNotifyIcons.Add(new CollectionContainer { Collection = notificationArea.PinnedIcons });
        pinnedNotifyIconsSource = new CollectionViewSource { Source = pinnedNotifyIcons };

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

    // private void NotifyIconToggleButton_OnClick(object sender, RoutedEventArgs e)
    // {
    //   if (NotifyIconToggleButton.IsChecked == true)
    //   {

    //     NotifyIcons.ItemsSource = allNotifyIconsSource.View;
    //   }
    //   else
    //   {
    //     NotifyIcons.ItemsSource = pinnedNotifyIconsSource.View;
    //   }
    // }

    // private void SetToggleVisibility()
    // {
    //   // if (!Settings.Instance.CollapseNotifyIcons) return;

    //   if (notificationArea.UnpinnedIcons.IsEmpty)
    //   {
    //     NotifyIconToggleButton.Visibility = Visibility.Collapsed;

    //     if (NotifyIconToggleButton.IsChecked == true)
    //     {
    //       NotifyIconToggleButton.IsChecked = false;
    //     }
    //   }
    //   else
    //   {
    //     NotifyIconToggleButton.Visibility = Visibility.Visible;
    //   }
    // }
  }
}