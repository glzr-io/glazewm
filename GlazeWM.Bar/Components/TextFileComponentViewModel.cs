using System;
using System.IO;
using System.Reactive.Linq;
using System.Threading;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.UserConfigs.Events;
using GlazeWM.Infrastructure;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Domain.Containers;
using System.Windows.Input;
using GlazeWM.Bar.Common;

namespace GlazeWM.Bar.Components
{
  public class TextFileComponentViewModel : ComponentViewModel
  {
    private readonly TextFileComponentConfig _baseConfig;
    private readonly IDisposable _disposable;
    private readonly Bus _bus = ServiceLocator.GetRequiredService<Bus>();
    private readonly ContainerService _containerService =
      ServiceLocator.GetRequiredService<ContainerService>();
    private readonly CommandParsingService _commandParsingService =
      ServiceLocator.GetRequiredService<CommandParsingService>();

    public string Text { get; set; } = "Loading...";

    public ICommand LeftClickCommand => new RelayCommand(OnLeftClick);
    public ICommand RightClickCommand => new RelayCommand(OnRightClick);

    public FileSystemWatcher Watcher { get; }

    public TextFileComponentViewModel(BarViewModel parentViewModel, TextFileComponentConfig baseConfig) : base(parentViewModel, baseConfig)
    {
      _baseConfig = baseConfig;

      var bus = ServiceLocator.GetRequiredService<Bus>();
      _disposable = bus.Events.OfType<UserConfigReloadedEvent>().Subscribe(_ => Dispose(true));

      Watcher = new FileSystemWatcher(Path.GetDirectoryName(_baseConfig.FilePath)!)
      {
        Filter = Path.GetFileName(_baseConfig.FilePath)!,
        EnableRaisingEvents = true,
        NotifyFilter = NotifyFilters.LastWrite
      };
      Watcher.Changed += OnFileChanged;
      Update();
    }

    public void OnLeftClick()
    {
      InvokeCommand(_baseConfig.LeftClickCommand);
    }

    public void OnRightClick()
    {
      InvokeCommand(_baseConfig.RightClickCommand);
    }

    private void InvokeCommand(string commandString)
    {
      if (string.IsNullOrEmpty(commandString))
        return;

      var subjectContainer = _containerService.FocusedContainer;

      var parsedCommand = _commandParsingService.ParseCommand(
        commandString,
        subjectContainer
      );

      // Use `dynamic` to resolve the command type at runtime and allow multiple dispatch.
      _bus.Invoke((dynamic)parsedCommand);
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

    private void OnFileChanged(object sender, FileSystemEventArgs e)
    {
      Update();
    }

    private void Update()
    {
      var numAttempts = 0;
      var sleepTime = 32;
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
}
