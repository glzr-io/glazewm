namespace GlazeWM.App.IpcServer.ServerMessages
{
  internal sealed record ClientResponseMessage<T>(
    /// <summary>
    /// The client message that this is in response to.
    /// </summary>
    string ClientMessage,

    /// <inheritdoc/>
    bool Success,

    /// <inheritdoc/>
    ServerMessageType MessageType,

    /// <inheritdoc/>
    T? Data,

    /// <inheritdoc/>
    string? Error
  ) : ServerMessage<T>(Success, MessageType, Data, Error);
}
