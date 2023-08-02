namespace GlazeWM.Application.IpcServer.Server
{
  internal record ServerMessage<T>(
    bool Success,

    /// <summary>
    /// The type of server message.
    /// </summary>
    ServerMessageType MessageType,

    /// <summary>
    /// The response or event data. This property is only present for messages where
    /// 'Success' is true.
    /// </summary>
    T? Data,

    /// <summary>
    /// The error message. This property is only present for messages where 'Success'
    /// is false.
    /// </summary>
    string? Error,

    /// <summary>
    /// The client message that this is in response to. This property is only present for
    /// 'ClientResponse' message types.
    /// </summary>
    string? ClientMessage);
}
