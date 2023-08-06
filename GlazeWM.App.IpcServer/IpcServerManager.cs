using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using GlazeWM.App.IpcServer.Server;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.Extensions.Logging;

namespace GlazeWM.App.IpcServer
{
  public sealed class IpcServerManager : IDisposable
  {
    private readonly Bus _bus;
    private readonly IpcMessageHandler _ipcMessageHandler;
    private readonly ILogger<IpcServerManager> _logger;

    /// <summary>
    /// The websocket server instance.
    /// </summary>
    private Server.IpcServer? _server { get; set; }

    private readonly Subject<bool> _serverKill = new();

    public IpcServerManager(
      Bus bus,
      IpcMessageHandler ipcMessageHandler,
      ILogger<IpcServerManager> logger)
    {
      _bus = bus;
      _ipcMessageHandler = ipcMessageHandler;
      _logger = logger;
    }

    /// <summary>
    /// Start the IPC server on specified port.
    /// </summary>
    public void StartServer(int port)
    {
      _server = new(port);
      _server.Start();

      // Start listening for messages.
      _server.Messages
        .TakeUntil(_serverKill)
        .Subscribe(clientMessage =>
        {
          var responseMessage = _ipcMessageHandler.GetResponseMessage(clientMessage);
          SendToSession(clientMessage.SessionId, responseMessage);
        });

      // Broadcast events to subscribed sessions.
      _bus.Events
        .TakeUntil(_serverKill)
        .Subscribe(@event =>
        {
          var subscribedSessionIds =
            _ipcMessageHandler.SubscribedSessions.GetValueOrDefault(
              @event.FriendlyName,
              new List<Guid>()
            );

          foreach (var sessionId in subscribedSessionIds)
          {
            var responseMessage = _ipcMessageHandler.ToEventMessage(@event);
            SendToSession(sessionId, responseMessage);
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

    /// <summary>
    /// Send text message to given session ID.
    /// </summary>
    private void SendToSession(Guid sessionId, string text)
    {
      var session = _server?.FindSession(sessionId) as IpcSession;
      session?.SendTextAsync(text);
    }

    public void Dispose()
    {
      if (_server?.IsDisposed != true)
        _server?.Dispose();
    }
  }
}
