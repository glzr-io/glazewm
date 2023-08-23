using System;
using System.Collections.Generic;
using System.Linq;
using System.Net.WebSockets;
using System.Reactive.Linq;
using System.Text;
using System.Text.Json;
using System.Text.RegularExpressions;
using System.Threading;
using System.Threading.Tasks;
using CommandLine;
using GlazeWM.App.IpcServer.ClientMessages;
using GlazeWM.App.IpcServer.ServerMessages;
using GlazeWM.Domain.Common;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Common;
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

    /// <summary>
    /// Used to subscribe to all possible event types at once.
    /// </summary>
    private const string SubscribeAllKeyword = "all";

    /// <summary>
    /// Allowed events to subscribe to via `subscribe` message.
    /// </summary>
    private static readonly List<string> SubscribableEvents = new()
    {
      SubscribeAllKeyword,
      DomainEvent.BindingModeChanged,
      DomainEvent.FocusChanged,
      DomainEvent.MonitorAdded,
      DomainEvent.MonitorRemoved,
      DomainEvent.TilingDirectionChanged,
      DomainEvent.UserConfigReloaded,
      DomainEvent.WorkspaceActivated,
      DomainEvent.WorkspaceDeactivated,
      InfraEvent.ApplicationExiting,
    };

    /// <summary>
    /// Dictionary of event names and WS connections subscribed to that event.
    /// </summary>
    /// TODO: Change to hash set.
    internal Dictionary<string, List<WebSocket>> EventSubscriptions =
      SubscribableEvents.ToDictionary(eventName => eventName, _ => new List<WebSocket>());

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

    /// <summary>
    /// Subscribe to events emitted on the bus and emit them to subscribed connections.
    /// </summary>
    internal void Init()
    {
      _bus.Events.Subscribe((@event) =>
      {
        var subscribedConnections = EventSubscriptions.GetValueOrDefault(
          @event.Type,
          new()
        );

        foreach (var ws in subscribedConnections)
        {
          var serverMessage = ToClientResponseMessage(
            success: true,
            messageType: ServerMessageType.EventSubscription,
            data: @event
          );

          _ = ws.SendAsync(
            serverMessage,
            WebSocketMessageType.Text,
            true,
            CancellationToken.None
          );
        }
      });
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

          // Handle close messages.
          if (received.MessageType == WebSocketMessageType.Close)
            await ws.CloseAsync(
              received.CloseStatus ?? WebSocketCloseStatus.NormalClosure,
              received.CloseStatusDescription,
              CancellationToken.None
            );

          // Ignore messages that aren't text.
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

      // TODO: Handle removal of event subscriptions.
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

        var data = parsedArgs.Value switch
        {
          InvokeCommandMessage commandMsg => HandleInvokeCommandMessage(commandMsg),
          SubscribeMessage subscribeMsg => HandleSubscribeMessage(subscribeMsg, ws),
          GetMonitorsMessage => _monitorService.GetMonitors(),
          GetWorkspacesMessage => _workspaceService.GetActiveWorkspaces(),
          GetWindowsMessage => _windowService.GetWindows(),
          _ => throw new Exception($"Invalid message '{message}'")
        };

        return ToClientResponseMessage(
          success: true,
          messageType: ServerMessageType.ClientResponse,
          data: data,
          clientMessage: message
        );
      }
      catch (Exception exception)
      {
        return ToClientResponseMessage(
          success: false,
          messageType: ServerMessageType.ClientResponse,
          data: null,
          error: exception.Message,
          clientMessage: message
        );
      }
    }

    private object HandleInvokeCommandMessage(InvokeCommandMessage message)
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

      return new { contextContainerId = contextContainer.Id };
    }

    private object HandleSubscribeMessage(SubscribeMessage message, WebSocket ws)
    {
      var eventNames = message.Events
        .Split(',')
        .Select(eventName => eventName.ToLowerInvariant());

      foreach (var eventName in eventNames)
      {
        if (!SubscribableEvents.Contains(eventName))
          throw new ArgumentException($"Invalid event '{eventName}'.");
      }

      // TODO: Does subscribe all need special handling?

      // var isSubscribeAll = eventNames.Contains(SubscribeAllKeyword);
      // var xx = isSubscribeAll ? SubscribableEvents : eventNames;

      foreach (var eventName in eventNames)
      {
        var subscribedConnections = EventSubscriptions.GetValueOrDefault(
          eventName,
          new()
        );

        subscribedConnections.Add(ws);
      }

      // TODO: Add field SubscriptionIds. Contains dictionary of subscription ids
      // to event names list.
      return new { subscriptionId = Guid.NewGuid() };
    }

    private ArraySegment<byte> ToClientResponseMessage(
      string clientMessage,
      bool success,
      object? data,
      string? error = null)
    {
      // Use `object` type so that the JSON serializer uses derived type.
      var serverMessage = new ClientResponseMessage<object>(
        ClientMessage: clientMessage,
        Success: success,
        MessageType: ServerMessageType.ClientResponse,
        Data: data,
        Error: error
      );

      var messageString = JsonParser.ToString(serverMessage, _serializeOptions);
      var messageBytes = Encoding.UTF8.GetBytes(messageString);

      return new ArraySegment<byte>(messageBytes);
    }

    private ArraySegment<byte> ToEventSubscriptionMessage(
      string subscriptionId,
      object? data,
      string? error = null)
    {
      // Use `object` type so that the JSON serializer uses derived type.
      var serverMessage = new EventSubscriptionMessage<object>(
        SubscriptionId: subscriptionId,
        Success: true,
        MessageType: ServerMessageType.EventSubscription,
        Data: data,
        Error: error
      );

      var messageString = JsonParser.ToString(serverMessage, _serializeOptions);
      var messageBytes = Encoding.UTF8.GetBytes(messageString);

      return new ArraySegment<byte>(messageBytes);
    }

    private ArraySegment<byte> MessageToBytes<T>(ServerMessage<T> serverMessage)
    {
      var messageString = JsonParser.ToString(serverMessage, _serializeOptions);
      var messageBytes = Encoding.UTF8.GetBytes(messageString);

      return new ArraySegment<byte>(messageBytes);
    }
  }
}
