using System;
using System.IO;
using System.Text.Json;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class LoadManagedWindowsHandler : ICommandHandler<LoadManagedWindowsCommand>
  {
    private readonly Bus _bus;

    public LoadManagedWindowsHandler(Bus bus)
    {
      _bus = bus;
    }

    public CommandResponse Handle(LoadManagedWindowsCommand command)
    {
      var managedWindowsPath = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile) + "/.glaze-wm/managedWindows.json";
      if (!File.Exists(managedWindowsPath))
      {
        return CommandResponse.Ok;
      }

      var json = File.ReadAllText(managedWindowsPath);
      foreach (var window in JsonSerializer.Deserialize<SerializeableWindow[]>(json))
      {
        var handle = (IntPtr)window.Handle;
        if (!WindowService.IsHandleVisible(handle))
          _bus.Invoke(new ManageWindowCommand(handle));
      }
      return CommandResponse.Ok;
    }
  }
}