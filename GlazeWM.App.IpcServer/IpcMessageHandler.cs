using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;
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
using GlazeWM.Infrastructure.Utils;
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
    /// Dictionary of event names and session IDs subscribed to that event.
    /// </summary>
    internal Dictionary<string, List<Guid>> SubscribedSessions = new();

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

    internal string GetResponseMessage(ClientMessage message)
    {
      var (sessionId, messageString) = message;

      _logger.LogDebug(
        "IPC message from session {Session}: {Message}.",
        sessionId,
        messageString
      );

      try
      {
        var messageParts = _messagePartsRegex.Matches(messageString)
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
          SubscribeMessage subscribeMsg => HandleSubscribeMessage(subscribeMsg, sessionId),
          GetMonitorsMessage => _monitorService.GetMonitors(),
          GetWorkspacesMessage => _workspaceService.GetActiveWorkspaces(),
          GetWindowsMessage => _windowService.GetWindows(),
          _ => throw new Exception($"Invalid message '{messageString}'")
        };

        return ToResponseMessage(
          success: true,
          data: data,
          clientMessage: messageString
        );
      }
      catch (Exception exception)
      {
        return ToResponseMessage<bool?>(
          success: false,
          data: null,
          clientMessage: messageString,
          error: exception.Message
        );
      }
    }

    private bool? HandleInvokeCommandMessage(InvokeCommandMessage message)
    {
      var contextContainer =
        _containerService.GetContainerById(Guid.Parse(message.ContextContainerId)) ??
        _containerService.FocusedContainer;

      var commandString = CommandParsingService.FormatCommand(message.Command);

      var command = _commandParsingService.ParseCommand(
        commandString,
        contextContainer
      );

      _bus.Invoke((dynamic)command);
      return null;
    }

    private bool? HandleSubscribeMessage(SubscribeMessage message, Guid sessionId)
    {
      foreach (var eventName in message.Events.Split(','))
      {
        if (SubscribedSessions.ContainsKey(eventName))
        {
          var sessionIds = SubscribedSessions.GetValueOrThrow(eventName);
          sessionIds.Add(sessionId);
          continue;
        }

        SubscribedSessions.Add(eventName, new() { sessionId });
      }

      return null;
    }

    private string ToResponseMessage<T>(
      bool success,
      T? data,
      string clientMessage,
      string? error = null)
    {
      var responseMessage = new ServerMessage<T>(
        Success: success,
        MessageType: ServerMessageType.ClientResponse,
        Data: data,
        Error: error,
        ClientMessage: clientMessage
      );

      return JsonParser.ToString(responseMessage, _serializeOptions);
    }

    internal string ToEventMessage(Event @event)
    {
      // Set type to `object` so that the JSON serializer uses derived `Event` type.
      var eventMessage = new ServerMessage<object>(
        Success: true,
        MessageType: ServerMessageType.SubscribedEvent,
        Data: @event,
        Error: null,
        ClientMessage: null
      );

      return JsonParser.ToString(eventMessage, _serializeOptions);
    }
  }
}
