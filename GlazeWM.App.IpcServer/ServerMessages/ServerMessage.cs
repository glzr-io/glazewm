namespace GlazeWM.App.IpcServer.ServerMessages
{
  internal abstract record ServerMessage<T>(
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
    string? Error);
}
