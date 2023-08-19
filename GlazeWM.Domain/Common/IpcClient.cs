using System;
using System.Net.WebSockets;
using System.Text;
using System.Text.Json;
using System.Threading;
using System.Threading.Tasks;

namespace GlazeWM.Domain.Common
{
  public class IpcClient : IDisposable
  {
    private readonly ClientWebSocket _ws = new();
    private readonly int _port;

    public IpcClient(int port)
    {
      _port = port;
    }

    public async Task ConnectAsync()
    {
      await _ws.ConnectAsync(
        new Uri($"ws://localhost:{_port}"),
        CancellationToken.None
      );
    }

    public async Task<JsonElement> ReceiveAsync()
    {
      var buffer = new byte[1024 * 4];

      // Continuously listen until a text message is received.
      while (true)
      {
        var result = await _ws.ReceiveAsync(
          new ArraySegment<byte>(buffer),
          CancellationToken.None
        );

        if (result.MessageType != WebSocketMessageType.Text)
          continue;

        var message = Encoding.UTF8.GetString(buffer, 0, result.Count);
        return ParseMessage(message);
      }
    }

    private async Task SendTextAsync(string message)
    {
      var messageBytes = Encoding.UTF8.GetBytes(message);

      await _ws.SendAsync(
        new ArraySegment<byte>(messageBytes),
        WebSocketMessageType.Text,
        true,
        CancellationToken.None
      );
    }

    public async Task<JsonElement> SendAndWaitReplyAsync(string message)
    {
      await SendTextAsync(message);
      return await ReceiveAsync();
    }

    /// <summary>
    /// Parse JSON in server message.
    /// </summary>
    private static JsonElement ParseMessage(string message)
    {
      var parsedMessage = JsonDocument.Parse(message).RootElement;
      var error = parsedMessage.GetProperty("error").GetString();

      if (error is not null)
        throw new Exception(error);

      return parsedMessage.GetProperty("data");
    }

    public async Task DisconnectAsync()
    {
      await _ws.CloseAsync(
        WebSocketCloseStatus.NormalClosure,
        null,
        CancellationToken.None
      );
    }

    public void Dispose()
    {
      _ws.Dispose();
    }
  }
}
