using System;
using System.Net;
using System.Reactive.Subjects;
using System.Text.Json;
using System.Text.Json.Serialization;
using GlazeWM.Infrastructure.Serialization;
using NetCoreServer;

namespace GlazeWM.Interprocess.Websocket
{
  internal sealed class WebsocketServer : WsServer
  {
    private readonly JsonSerializerOptions _serializerOptions = new()
    {
      WriteIndented = true,
      PropertyNamingPolicy = new SnakeCaseNamingPolicy(),
      Converters =
      {
        new JsonStringEnumConverter()
      }
    };

    public readonly Subject<WebsocketMessage> MessageReceived = new();

    public WebsocketServer() : base(IPAddress.Any, 1337)
    {
    }

    public bool SendToAll<T>(T payload)
    {
      var json = JsonSerializer.Serialize(payload, _serializerOptions);

      return MulticastText(json);
    }

    public bool SendToSession<T>(T payload, Guid sessionId)
    {
      var session = FindSession(sessionId);

      if (session is not WebsocketSession websocketSession)
      {
        return false;
      }

      var json = JsonSerializer.Serialize(payload, _serializerOptions);

      return websocketSession.SendTextAsync(json);
    }

    protected override TcpSession CreateSession()
    {
      return new WebsocketSession(this);
    }

    protected override void Dispose(bool disposingManagedResources)
    {
      if (disposingManagedResources)
      {
        MessageReceived.Dispose();
      }

      base.Dispose(disposingManagedResources);
    }
  }
}
