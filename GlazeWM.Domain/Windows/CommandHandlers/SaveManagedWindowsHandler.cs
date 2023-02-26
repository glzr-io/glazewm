using System;
using System.IO;
using System.Linq;
using System.Text.Json;
using GlazeWM.Domain.Windows.Commands;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Domain.Windows.CommandHandlers
{
  internal sealed class SaveManagedWindowsHandler : ICommandHandler<SaveManagedWindowsCommand>
  {
    private readonly WindowService _windowService;
    private readonly ILogger<SaveManagedWindowsHandler> _logger;

    public SaveManagedWindowsHandler(WindowService windowService, ILogger<SaveManagedWindowsHandler> logger
     )
    {
      _windowService = windowService;
      _logger = logger;
    }

    public CommandResponse Handle(SaveManagedWindowsCommand command)
    {
      var windows = _windowService.GetWindows();
      var serializableWindows = windows.Select(window => new SerializeableWindow((long)window.Handle, WindowService.GetTitleOfHandle(window.Handle)));
      var json = JsonSerializer.Serialize(serializableWindows);
      _logger.LogDebug("Windows as JSON: {Json}", json);
      File.WriteAllTextAsync(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile) + "./.glaze-wm/managedWindows.json", json);
      return CommandResponse.Ok;
    }
  }
}
