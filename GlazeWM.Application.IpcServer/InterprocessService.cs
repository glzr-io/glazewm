using System;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using System.Text.Json;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Application.IpcServer.Websocket;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Application.IpcServer
{
  // TODO: Rename class to `ServerManager`.
  public sealed class InterprocessService : IDisposable
  {
    private readonly Bus _bus;
    private readonly ILogger<InterprocessService> _logger;
    private readonly UserConfigService _userConfigService;

    /// <summary>
    /// The websocket server instance.
    /// </summary>
    private WebsocketServer _server { get; set; }

    private readonly Subject<bool> _serverKill = new();

    private readonly JsonSerializerOptions _serializeOptions =
      JsonParser.OptionsFactory((options) =>
        options.Converters.Add(new JsonContainerConverter())
      );

    public InterprocessService(
      Bus bus,
      ILogger<InterprocessService> logger,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _logger = logger;
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

      _logger.LogDebug("Started IPC server on port {Port}.", port);
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
      _logger.LogDebug("Stopped IPC server on port {Port}.", _server.Port);
    }

    private void HandleMessage(WebsocketMessage message)
    {
      // TODO
      _logger.LogDebug(
        "IPC message from session {Session}: {Text}.",
        message.SessionId,
        message.Text
      );
    }

    private void BroadcastEvent(Event @event)
    {
      var eventJson = JsonParser.ToString((dynamic)@event, _serializeOptions);
      _server.MulticastText(eventJson);
    }

    public void Dispose()
    {
      if (_server?.IsDisposed != true)
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
