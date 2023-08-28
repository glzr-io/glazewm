using System;

namespace GlazeWM.App.IpcServer.ServerMessages
{
  internal sealed record EventSubscriptionMessage<T>(
    /// <summary>
    /// ID of the subscription.
    /// </summary>
    Guid SubscriptionId,

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
