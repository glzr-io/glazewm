using System;
using System.Collections.Generic;
using System.Net.WebSockets;
using System.Threading;
using GlazeWM.App.IpcServer.Server;
using GlazeWM.Infrastructure.Bussing;
using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Hosting;
using Microsoft.Extensions.Logging;

namespace GlazeWM.App.IpcServer
{
  public sealed class IpcServerStartup : IDisposable
  {
    private readonly Bus _bus;
    private readonly IpcMessageHandler _ipcMessageHandler;
    private readonly ILogger<IpcServerStartup> _logger;

    private List<WebSocket> _connections = new();

    public IpcServerStartup(
      Bus bus,
      IpcMessageHandler ipcMessageHandler,
      ILogger<IpcServerStartup> logger)
    {
      _bus = bus;
      _ipcMessageHandler = ipcMessageHandler;
      _logger = logger;
    }

    /// <summary>
    /// Start the IPC server on specified port.
    /// </summary>
    public void Run(int port)
    {
      var builder = WebApplication.CreateBuilder();
      builder.WebHost.UseUrls($"http://localhost:{port}");

      var app = builder.Build();
      app.UseWebSockets();

      app.Use(async (context, next) =>
      {
        if (!context.WebSockets.IsWebSocketRequest)
          await next();

        using var ws = await context.WebSockets.AcceptWebSocketAsync();

        _connections.Add(ws);

        var buffer = new byte[1024];
        while (!ws.CloseStatus.HasValue)
        {
          var received = await ws.ReceiveAsync(
            new ArraySegment<byte>(buffer),
            CancellationToken.None
          );

          _ipcMessageHandler.Handle(received, buffer);
        }

        await ws.CloseAsync(
          ws.CloseStatus.Value,
          ws.CloseStatusDescription,
          CancellationToken.None
        );
      });

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
