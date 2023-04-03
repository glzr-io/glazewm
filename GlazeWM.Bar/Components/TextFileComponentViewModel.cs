using System;
using System.IO;
using System.Reactive.Linq;
using System.Threading;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Events;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Bar.Components;

public class TextFileComponentViewModel : ComponentViewModel
{
  private readonly TextFileConfig _baseConfig;
  private readonly IDisposable _disposable;

  public string Text { get; set; } = "Loading...";

  public FileSystemWatcher Watcher { get; }

  public TextFileComponentViewModel(BarViewModel parentViewModel, TextFileConfig baseConfig) : base(parentViewModel, baseConfig)
  {
    _baseConfig = baseConfig;
    ServiceLocator.GetRequiredService<ILogger<TextFileComponentViewModel>>();

    var bus = ServiceLocator.GetRequiredService<Bus>();
    _disposable = bus.Events.OfType<UserConfigReloadedEvent>().Subscribe(_ => Dispose(true));

    Watcher = new FileSystemWatcher(Path.GetDirectoryName(_baseConfig.FilePath)!);
    Watcher.Filter = Path.GetFileName(_baseConfig.FilePath)!;
    Watcher.EnableRaisingEvents = true;
    Watcher.NotifyFilter = NotifyFilters.LastWrite;
    Watcher.Changed += OnFileChanged;
    Update();
  }

  protected override void Dispose(bool disposing)
  {
    if (disposing)
    {
      Watcher.Dispose();
      _disposable.Dispose();
    }

    base.Dispose(disposing);
  }

  private void OnFileChanged(object sender, FileSystemEventArgs e) => Update();

  private void Update()
  {
    int numAttempts = 0;
    int sleepTime = 32;
    const int maxRetries = 6;

    try
    {
      Watcher.EnableRaisingEvents = false;
      while (true)
      {
        try
        {
          Text = File.ReadAllText(_baseConfig.FilePath);
          OnPropertyChanged(nameof(Text));
          break;
        }
        catch (Exception) when (numAttempts < maxRetries)
        {
          numAttempts++;
          Thread.Sleep(sleepTime);
          sleepTime *= 2;
        }
      }
    }
    finally
    {
      Watcher.EnableRaisingEvents = true;
    }
  }
}
