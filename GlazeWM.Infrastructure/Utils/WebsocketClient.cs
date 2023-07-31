using System;
using System.Net.Sockets;
using System.Text;
using System.Threading;
using NetCoreServer;

namespace GlazeWM.Infrastructure.Utils
{
  public class ChatClient : WsClient
  {
    public ChatClient(string address, int port) : base(address, port) { }

    public void DisconnectAndStop()
    {
      _stop = true;
      CloseAsync(1000);
      while (IsConnected)
        Thread.Yield();
    }

    public override void OnWsConnecting(HttpRequest request)
    {
      request.SetBegin("GET", "/")
        .SetHeader("Host", "localhost")
        .SetHeader("Origin", "http://localhost")
        .SetHeader("Upgrade", "websocket")
        .SetHeader("Connection", "Upgrade")
        .SetHeader("Sec-WebSocket-Key", Convert.ToBase64String(WsNonce))
        .SetHeader("Sec-WebSocket-Protocol", "chat, superchat")
        .SetHeader("Sec-WebSocket-Version", "13")
        .SetBody();
    }

    public override void OnWsConnected(HttpResponse response)
    {
      Console.WriteLine($"Chat WebSocket client connected a new session with Id {Id}");
    }

    public override void OnWsDisconnected()
    {
      Console.WriteLine($"Chat WebSocket client disconnected a session with Id {Id}");
    }

    public override void OnWsReceived(byte[] buffer, long offset, long size)
    {
      Console.WriteLine($"Incoming: {Encoding.UTF8.GetString(buffer, (int)offset, (int)size)}");
    }

    protected override void OnDisconnected()
    {
      base.OnDisconnected();

      Console.WriteLine($"Chat WebSocket client disconnected a session with Id {Id}");

      // Wait for a while...
      Thread.Sleep(1000);

      // Try to connect again
      if (!_stop)
        ConnectAsync();
    }

    protected override void OnError(SocketError error)
    {
      Console.WriteLine($"Chat WebSocket client caught an error with code {error}");
    }

    private bool _stop;
  }

  public class WebsocketClient
  {
    public void Send(string message)
    {
      // WebSocket server address
      const string address = "127.0.0.1";

      // WebSocket server port
      const int port = 61423;

      Console.WriteLine($"WebSocket server address: {address}");
      Console.WriteLine($"WebSocket server port: {port}");

      Console.WriteLine();

      // Create a new TCP chat client
      var client = new ChatClient(address, port);

      // Connect the client
      Console.Write("Client connecting...");
      client.ConnectAsync();
      Console.WriteLine("Done!");

      Console.WriteLine("Press Enter to stop the client or '!' to reconnect the client...");

      // Perform text input
      for (; ; )
      {
        var line = Console.ReadLine();
        if (string.IsNullOrEmpty(line))
          break;

        // Disconnect the client
        if (line == "!")
        {
          Console.Write("Client disconnecting...");
          client.DisconnectAsync();
          Console.WriteLine("Done!");
          continue;
        }

        // Send the entered text to the chat server
        client.SendTextAsync(line);
      }

      // Disconnect the client
      Console.Write("Client disconnecting...");
      client.DisconnectAndStop();
      Console.WriteLine("Done!");
    }
  }
}
