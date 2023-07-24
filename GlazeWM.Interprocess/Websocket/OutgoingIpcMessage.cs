using System;

namespace GlazeWM.Interprocess.Websocket
{
  internal sealed class OutgoingIpcMessage<T>
  {
    public IpcPayloadType PayloadType { get; }
    public T Payload { get; }

    public OutgoingIpcMessage(IpcPayloadType payloadType, T payload)
    {
      PayloadType = payloadType;
      Payload = payload;
    }
  }
}
