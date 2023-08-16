using System;
using System.Net.WebSockets;
using System.Text;
using System.Threading;
using System.Threading.Tasks;

namespace GlazeWM.Infrastructure.Utils
{
  public class WebSocketClient
  {
    private readonly ClientWebSocket _ws = new();
    private readonly string _address;
    private readonly int _port;

    public WebSocketClient(string address, int port)
    {
      _address = address;
      _port = port;
    }

    public WebSocketClient(int port)
    {
      _address = "localhost";
      _port = port;
    }

    public async Task ConnectAsync(CancellationToken cancellationToken)
    {
      await _ws.ConnectAsync(
        new Uri($"ws://{_address}:{_port}"),
        cancellationToken
      );
    }

    public async Task<string> ReceiveTextAsync(CancellationToken cancellationToken)
    {
      var buffer = new byte[1024 * 4];

      // Continuously listen until a text message is received.
      while (true)
      {
        var result = await _ws.ReceiveAsync(
          new ArraySegment<byte>(buffer),
          cancellationToken
        );

        if (result.MessageType != WebSocketMessageType.Text)
          continue;

        return Encoding.UTF8.GetString(buffer, 0, result.Count);
      }
    }

    public async Task SendTextAsync(string message, CancellationToken cancellationToken)
    {
      var messageBytes = Encoding.UTF8.GetBytes(message);

      await _ws.SendAsync(
        new ArraySegment<byte>(messageBytes),
        WebSocketMessageType.Text,
        true,
        cancellationToken
      );
    }

    public async Task DisconnectAsync(CancellationToken cancellationToken)
    {
      await _ws.CloseAsync(WebSocketCloseStatus.NormalClosure, null, cancellationToken);
    }
  }
}
