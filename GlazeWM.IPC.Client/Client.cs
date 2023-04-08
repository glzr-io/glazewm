using System.Text.Json;
using GlazeWM.IPC.Client.Messages;
using NetMQ;
using NetMQ.Sockets;

namespace GlazeWM.IPC.Client;

/// <summary>
/// Client that connects to the IPC server.
/// </summary>
public class Client : IDisposable
{
  private readonly PublisherSocket _publisher;
  private bool _disposed;
  
  /// <summary>
  /// Constructor.
  /// </summary>
  /// <param name="port">Port used to connect to the localhost client.</param>
  public Client(int port)
  {
    _publisher = new PublisherSocket();
    _publisher.Connect($"tcp://localhost:{port}");
  }

  ~Client() => Dispose();

  /// <inheritdoc />
  public void Dispose()
  {
    if (_disposed)
      return;
    
    _disposed = true;
    _publisher.Dispose();
    GC.SuppressFinalize(this);
  }

  /// <summary>
  /// Sends a message that updates the state of a given label.
  /// </summary>
  /// <param name="labelId">Unique identifier for the label; matching 'label_id' in config.</param>
  /// <param name="message">Instructions to update the component.</param>
  /// <remarks>This is non-blocking.</remarks>
  public void SendIpcComponentUpdate(string labelId, UpdateIpcComponent message)
  {
    _publisher.SendMoreFrame(labelId)
      .SendFrame(JsonSerializer.Serialize(message));
  }
}
