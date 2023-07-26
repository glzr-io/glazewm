using CommandLine;
using GlazeWM.Application.IpcServer.Messages;
using GlazeWM.Application.IpcServer.Server;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Application.IpcServer
{
  public sealed class IpcMessageHandler
  {
    private readonly Bus _bus;
    private readonly CommandParsingService _commandParsingService;
    private readonly ContainerService _containerService;
    private readonly ILogger<IpcMessageHandler> _logger;
    private readonly MonitorService _monitorService;
    private readonly WorkspaceService _workspaceService;
    private readonly WindowService _windowService;

    public IpcMessageHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      ContainerService containerService,
      ILogger<IpcMessageHandler> logger,
      MonitorService monitorService,
      WorkspaceService workspaceService,
      WindowService windowService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _containerService = containerService;
      _logger = logger;
      _monitorService = monitorService;
      _workspaceService = workspaceService;
      _windowService = windowService;
    }

    public string GetResponse(IncomingIpcMessage message)
    {
      _logger.LogDebug(
        "IPC message from session {Session}: {Text}.",
        message.SessionId,
        message.Text
      );

      var ipcMessage = message.Text.Split(" ");

      return Parser.Default.ParseArguments<
        InvokeCommandMessage,
        SubscribeMessage,
        GetContainersMessage,
        GetMonitorsMessage,
        GetWorkspacesMessage,
        GetWindowsMessage
      >(ipcMessage).MapResult(
        (InvokeCommandMessage message) => HandleInvokeCommandMessage(message),
        (SubscribeMessage message) => 1,
        (GetContainersMessage message) => 1,
        (GetMonitorsMessage message) => 1,
        (GetWorkspacesMessage message) => 1,
        (GetWindowsMessage message) => 1,
        _ => 1
      );
    }

    public string HandleInvokeCommandMessage(InvokeCommandMessage message)
    {
      var contextContainer = _containerService.GetContainerById(
        message.ContextContainerId
      );

      var command = _commandParsingService.ParseCommand(
        message.Command,
        contextContainer
      );

      _bus.Invoke((dynamic)command);
      return string.Empty;
    }
  }
}
