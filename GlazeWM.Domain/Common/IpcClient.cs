using System;
using System.Net.WebSockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace GlazeWM.Domain.Common
{
  public class IpcClient
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

    public async Task<string> ReceiveTextAsync()
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

        return Encoding.UTF8.GetString(buffer, 0, result.Count);
      }
    }

    public async Task SendTextAsync(string message)
    {
      var messageBytes = Encoding.UTF8.GetBytes(message);

      await _ws.SendAsync(
        new ArraySegment<byte>(messageBytes),
        WebSocketMessageType.Text,
        true,
        CancellationToken.None
      );
    }

    public async Task SendTextAndWaitReplyAsync(string message)
    {
      var messageBytes = Encoding.UTF8.GetBytes(message);

      await _ws.SendAsync(
        new ArraySegment<byte>(messageBytes),
        WebSocketMessageType.Text,
        true,
        CancellationToken.None
      );
    }

    public async Task DisconnectAsync()
    {
      await _ws.CloseAsync(
        WebSocketCloseStatus.NormalClosure,
        null,
        CancellationToken.None
      );
    }
  }
}
