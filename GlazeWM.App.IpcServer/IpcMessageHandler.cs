using System;
using System.Linq;
using System.Net.WebSockets;
using System.Reactive.Linq;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using CommandLine;
using GlazeWM.App.IpcServer.Messages;
using GlazeWM.App.IpcServer.Server;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Serialization;
using Microsoft.Extensions.Logging;

namespace GlazeWM.App.IpcServer
{
  public sealed class IpcMessageHandler
  {
    private readonly Bus _bus;
    private readonly CommandParsingService _commandParsingService;
    private readonly ContainerService _containerService;
    private readonly ILogger<IpcMessageHandler> _logger;
    private readonly MonitorService _monitorService;
    private readonly WorkspaceService _workspaceService;
    private readonly WindowService _windowService;

    private readonly JsonSerializerOptions _serializeOptions =
      JsonParser.OptionsFactory((options) =>
      {
        options.PropertyNamingPolicy = JsonNamingPolicy.CamelCase;
        options.Converters.Add(new JsonContainerConverter());
      });

    /// <summary>
    /// Matches words separated by spaces when not surrounded by double quotes.
    /// Example: "a \"b c\" d" -> ["a", "\"b c\"", "d"]
    /// </summary>
    private static readonly Regex _messagePartsRegex = new("(\".*?\"|\\S+)");

    public IpcMessageHandler(
      Bus bus,
      CommandParsingService commandParsingService,
      ContainerService containerService,
      ILogger<IpcMessageHandler> logger,
      MonitorService monitorService,
      WorkspaceService workspaceService,
      WindowService windowService)
    {
      _bus = bus;
      _commandParsingService = commandParsingService;
      _containerService = containerService;
      _logger = logger;
      _monitorService = monitorService;
      _workspaceService = workspaceService;
      _windowService = windowService;
    }

    internal async Task Handle(WebSocket ws)
    {
      var buffer = new byte[1024];

      while (ws.State == WebSocketState.Open)
      {
        try
        {
          var received = await ws.ReceiveAsync(
            new ArraySegment<byte>(buffer),
            CancellationToken.None
          );

          if (received.MessageType != WebSocketMessageType.Text)
            continue;

          var clientMessage = Encoding.UTF8.GetString(buffer, 0, received.Count);
          var serverMessage = GetResponseMessage(clientMessage, ws);

          await ws.SendAsync(
            serverMessage,
            WebSocketMessageType.Text,
            true,
            CancellationToken.None
          );
        }
        catch
        {
        }
      }
    }

    private ArraySegment<byte> GetResponseMessage(string message, WebSocket ws)
    {
      _logger.LogDebug("IPC message received: {Message}.", message);

      try
      {
        var messageParts = _messagePartsRegex.Matches(message)
          .Select(match => match.Value)
          .Where(match => match is not null);

        var parsedArgs = Parser.Default.ParseArguments<
          InvokeCommandMessage,
          SubscribeMessage,
          GetMonitorsMessage,
          GetWorkspacesMessage,
          GetWindowsMessage
        >(messageParts);

        object? data = parsedArgs.Value switch
        {
          InvokeCommandMessage commandMsg => HandleInvokeCommandMessage(commandMsg),
          SubscribeMessage subscribeMsg => HandleSubscribeMessage(subscribeMsg, ws),
          GetMonitorsMessage => _monitorService.GetMonitors(),
          GetWorkspacesMessage => _workspaceService.GetActiveWorkspaces(),
          GetWindowsMessage => _windowService.GetWindows(),
          _ => throw new Exception($"Invalid message '{message}'")
        };

        return ToServerMessage(
          success: true,
          messageType: ServerMessageType.ClientResponse,
          data: data,
          clientMessage: message
        );
      }
      catch (Exception exception)
      {
        return ToServerMessage(
          success: false,
          messageType: ServerMessageType.ClientResponse,
          data: null,
          error: exception.Message,
          clientMessage: message
        );
      }
    }

    private bool? HandleInvokeCommandMessage(InvokeCommandMessage message)
    {
      var contextContainer =
        message.ContextContainerId is not null
          ? _containerService.GetContainerById(Guid.Parse(message.ContextContainerId))
          : _containerService.FocusedContainer;

      var commandString = CommandParsingService.FormatCommand(message.Command);

      var command = _commandParsingService.ParseCommand(
        commandString,
        contextContainer
      );

      _bus.Invoke((dynamic)command);
      return null;
    }

    private bool? HandleSubscribeMessage(SubscribeMessage message, WebSocket ws)
    {
      var eventNames = message.Events.Split(',');

      _bus.Events
        .TakeWhile((_) => ws.State == WebSocketState.Open)
        .Where(@event => eventNames.Contains(@event.FriendlyName))
        .Subscribe((@event) =>
        {
          var serverMessage = ToServerMessage(
            success: true,
            messageType: ServerMessageType.SubscribedEvent,
            data: @event
          );

          _ = ws.SendAsync(
            serverMessage,
            WebSocketMessageType.Text,
            true,
            CancellationToken.None
          );
        });

      return null;
    }

    private ArraySegment<byte> ToServerMessage(
      bool success,
      ServerMessageType messageType,
      object? data,
      string? error = null,
      string? clientMessage = null)
    {
      // Use `object` type so that the JSON serializer uses derived type.
      var serverMessage = new ServerMessage<object>(
        Success: success,
        MessageType: messageType,
        Data: data,
        Error: error,
        ClientMessage: clientMessage
      );

      var messageString = JsonParser.ToString(serverMessage, _serializeOptions);
      var messageBytes = Encoding.UTF8.GetBytes(messageString);

      return new ArraySegment<byte>(messageBytes);
    }
  }
}
