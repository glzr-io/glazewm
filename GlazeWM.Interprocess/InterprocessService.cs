using System;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Interprocess.Modules;
using GlazeWM.Interprocess.Websocket;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Interprocess
{
  public sealed class InterprocessService : IDisposable
  {
    private readonly Bus _bus;
    private readonly ILogger<InterprocessService> _logger;
    private readonly JsonService _jsonService;
    private readonly UserConfigService _userConfigService;

    /// <summary>
    /// The websocket server instance.
    /// </summary>
    private readonly WebsocketServer _server;

    private readonly Subject<bool> _serverKill = new();

    public InterprocessService(
      Bus bus,
      ILogger<InterprocessService> logger,
      JsonService _jsonService,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _logger = logger;
      _jsonService = _jsonService;
      _userConfigService = userConfigService;
    }

    /// <summary>
    /// Start the IPC server on user-specified port.
    /// </summary>
    public void StartIpcServer()
    {
      var port = _userConfigService.GeneralConfig.IpcServerPort;
      _server = new(port);

      // Start listening for messages.
      _server.Start();
      _server.Messages
        .TakeUntil(_serverKill)
        .Subscribe(message => HandleMessage(message));

      // Broadcast events to all sessions.
      _bus.Events
        .TakeUntil(_serverKill)
        .Subscribe(@event => BroadcastEvent(@event));

      _logger.LogDebug("Started IPC server on port {port}.", port);
    }

    /// <summary>
    /// Kill the IPC server.
    /// </summary>
    public void StopIpcServer()
    {
      if (_server is null)
        return;

      _serverKill.OnNext(true);
      _server.Stop();
      _logger.LogDebug("Stopped IPC server on port {port}.", _server.Port);
    }

    private async void HandleMessage(WebsocketMessage message)
    {
      // TODO
    }

    private async void BroadcastEvent(Event @event)
    {
      var eventJson = _jsonService.Serialize(@event);
      await _server.MulticastText(eventJson);
    }

    public void Dispose()
    {
      if (!_server?.IsDisposed)
        _server.Dispose();
    }
  }
}

/**
Example usage:
const client = new GwmClient({ port });
client.on(GwmEvent.ALL, () => {})
client.on([GwmEvent.WORKSPACE_FOCUSED], () => {})

const targetContainerId = (await client.getWorkspaces())[0].id;
client.sendCommand('move workspace left', { targetContainerId })

type Direction = 'left' | 'right' | 'up' | 'down';
type MoveWorkspaceCommand = `move workspace ${Direction}`;
type GwmCommand = MoveWorkspaceCommand | MoveWindowCommand | ...;
**/
