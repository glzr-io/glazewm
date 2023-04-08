using System;
using System.Diagnostics;
using System.Threading;
using System.Threading.Tasks;
using NetMQ;
using NetMQ.Sockets;

namespace GlazeWM.Infrastructure.Services;

/// <summary>
/// Implementation of a TCP based IPC implementation.
/// This is intended to be used as a singleton.
/// </summary>
public class IpcService : IDisposable
{
  /// <summary>
  /// Called when a message is received.
  /// </summary>
  public event MessageReceived OnMessageReceived;

  private readonly SubscriberSocket _subscriber;
  private bool _isRunning = true;

  public IpcService(IIpcConfigContainer configContainer)
  {
    var ipcConfig = configContainer.Config;
    _subscriber = new SubscriberSocket();
    _subscriber.SubscribeToAnyTopic();
    _subscriber.Bind($"tcp://localhost:{ipcConfig.NetMqPort}");
    _ = Task.Run(RunServer);
  }

  private void RunServer()
  {
    while (_isRunning)
    {
      var messageTopicReceived = _subscriber.ReceiveFrameString();
      var messageReceived = _subscriber.ReceiveFrameString();
      OnMessageReceived?.Invoke(messageTopicReceived, messageReceived);
    }
  }

  ~IpcService() => Dispose();

  public void Dispose()
  {
    _isRunning = false;
    _subscriber?.Dispose();
    GC.SuppressFinalize(this);
  }

  public delegate void MessageReceived(string topic, string data);
}

public interface IIpcConfig
{
  /// <summary>
  /// Port used for IPC using NetMQ; a ZeroMQ TCP implementation.
  /// </summary>
  public int NetMqPort { get; }
}

public interface IIpcConfigContainer
{
  /// <summary>
  /// Gets the config used for IPC.
  /// </summary>
  public IIpcConfig Config { get; }
}
