using System;
using System.Diagnostics;
using System.Reactive.Linq;
using System.Text.Json;
using System.Windows;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Events;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Services;
using GlazeWM.IPC.Client.Messages;

namespace GlazeWM.Bar.Components;

public class IpcComponentViewModel : ComponentViewModel
{
  private IpcComponentConfig Config => _componentConfig as IpcComponentConfig;
  private readonly IpcService _ipcService;
  private readonly IDisposable _disposable;

  public string Text { get; set; }

  public IpcComponentViewModel(BarViewModel parentViewModel, IpcComponentConfig config) : base(parentViewModel, (BarComponentConfig)config.Clone())
  {
    // Note: In constructor we clone parameter; as we want separate one for each bar,
    // to ensure we can properly call OnPropertyChanged with multiple monitors.
    _ipcService = ServiceLocator.GetRequiredService<IpcService>();

    var bus = ServiceLocator.GetRequiredService<Bus>();
    _ipcService.OnMessageReceived += OnIpcMessageReceived;
    _disposable = bus.Events.OfType<UserConfigReloadedEvent>().Subscribe(_ => Dispose(true));
  }

  private void OnIpcMessageReceived(string topic, string data)
  {
    var conf = Config;
    if (!topic.Equals(conf.LabelId, StringComparison.Ordinal))
      return;

    var deserialized = JsonSerializer.Deserialize<UpdateIpcComponent>(data);
    if (Application.Current == null) // on shutdown
      return;

    Application.Current.Dispatcher.InvokeAsync(() =>
    {
      // No way to do this with high perf, since we can't ref a property.. 
      if (!string.Equals(Text, deserialized.Text, StringComparison.Ordinal))
      {
        Text = deserialized.Text;
        OnPropertyChanged(nameof(Text));
      }

      if (deserialized.Margin != null &&
          !string.Equals(conf.Margin, deserialized.Margin, StringComparison.Ordinal))
      {
        conf.Margin = deserialized.Margin;
        OnPropertyChanged(nameof(Margin));
      }

      if (deserialized.Background != null &&
          !string.Equals(conf.Background, deserialized.Background, StringComparison.Ordinal))
      {
        conf.Background = deserialized.Background;
        OnPropertyChanged(nameof(Background));
      }

      if (deserialized.Foreground != null &&
          !string.Equals(conf.Foreground, deserialized.Foreground, StringComparison.Ordinal))
      {
        conf.Foreground = deserialized.Foreground;
        OnPropertyChanged(nameof(Foreground));
      }

      if (deserialized.FontFamily != null &&
          !string.Equals(conf.FontFamily, deserialized.FontFamily, StringComparison.Ordinal))
      {
        conf.FontFamily = deserialized.FontFamily;
        OnPropertyChanged(nameof(FontFamily));
      }

      if (deserialized.FontWeight != null &&
          !string.Equals(conf.FontWeight, deserialized.FontFamily, StringComparison.Ordinal))
      {
        conf.FontWeight = deserialized.FontWeight;
        OnPropertyChanged(nameof(FontWeight));
      }

      if (deserialized.FontSize != null &&
          !string.Equals(conf.FontSize, deserialized.FontFamily, StringComparison.Ordinal))
      {
        conf.FontSize = deserialized.FontSize;
        OnPropertyChanged(nameof(FontSize));
      }

      if (deserialized.BorderColor != null && !string.Equals(conf.BorderColor, deserialized.FontFamily,
            StringComparison.Ordinal))
      {
        conf.BorderColor = deserialized.BorderColor;
        OnPropertyChanged(nameof(BorderColor));
      }

      if (deserialized.BorderRadius != null && !string.Equals(conf.BorderRadius, deserialized.FontFamily,
            StringComparison.Ordinal))
      {
        conf.BorderRadius = deserialized.BorderRadius;
        OnPropertyChanged(nameof(BorderRadius));
      }

      if (deserialized.BorderWidth != null && !string.Equals(conf.BorderWidth, deserialized.FontFamily,
            StringComparison.Ordinal))
      {
        conf.BorderWidth = deserialized.BorderWidth;
        OnPropertyChanged(nameof(BorderWidth));
      }

      if (deserialized.Padding != null &&
          !string.Equals(conf.BorderWidth, deserialized.FontFamily, StringComparison.Ordinal))
      {
        conf.Padding = deserialized.Padding;
        OnPropertyChanged(nameof(Padding));
      }
    });
  }

  protected override void Dispose(bool disposing)
  {
    if (disposing)
    {
      _ipcService.OnMessageReceived -= OnIpcMessageReceived;
      _disposable.Dispose();
    }

    base.Dispose(disposing);
  }
}
