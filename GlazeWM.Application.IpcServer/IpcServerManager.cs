using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Application.IpcServer
{
  public sealed class IpcServerManager : IDisposable
  {
    private readonly Bus _bus;
    private readonly IpcMessageHandler _ipcMessageHandler;
    private readonly ILogger<IpcServerManager> _logger;
    private readonly UserConfigService _userConfigService;

    /// <summary>
    /// The websocket server instance.
    /// </summary>
    private Server.IpcServer _server { get; set; }

    private readonly Subject<bool> _serverKill = new();

    public IpcServerManager(
      Bus bus,
      IpcMessageHandler ipcMessageHandler,
      ILogger<IpcServerManager> logger,
      UserConfigService userConfigService)
    {
      _bus = bus;
      _ipcMessageHandler = ipcMessageHandler;
      _logger = logger;
      _userConfigService = userConfigService;
    }

    /// <summary>
    /// Start the IPC server on user-specified port.
    /// </summary>
    public void StartServer()
    {
      var port = _userConfigService.GeneralConfig.IpcServerPort;
      _server = new(port);

      // Start listening for messages.
      _server.Start();
      _server.Messages
        .TakeUntil(_serverKill)
        .Subscribe(clientMessage =>
        {
          var responseMessage = _ipcMessageHandler.GetResponseMessage(clientMessage);
          var session = _server.FindSession(clientMessage.SessionId);
          session.SendAsync(responseMessage);
        });

      // Broadcast events to subscribed sessions.
      _bus.Events
        .TakeUntil(_serverKill)
        .Subscribe(@event =>
        {
          var subscribedSessionIds = new List<Guid>();

          _ipcMessageHandler.SubscribedSessions.TryGetValue(
            @event.FriendlyName,
            out subscribedSessionIds
          );

          foreach (var sessionId in subscribedSessionIds)
          {
            var session = _server.FindSession(sessionId);
            var responseMessage = _ipcMessageHandler.ToEventMessage(@event);
            session.SendAsync(responseMessage);
          }
        });

      _logger.LogDebug("Started IPC server on port {Port}.", port);
    }

    /// <summary>
    /// Kill the IPC server.
    /// </summary>
    public void StopServer()
    {
      if (_server is null)
        return;

      _serverKill.OnNext(true);
      _server.Stop();
      _logger.LogDebug("Stopped IPC server on port {Port}.", _server.Port);
    }

    public void Dispose()
    {
      if (_server?.IsDisposed != true)
        _server.Dispose();
    }
  }
}
