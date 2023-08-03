using System;
using System.Collections.Generic;
using System.Linq;
using System.Text.Json;
using System.Text.RegularExpressions;
using CommandLine;
using GlazeWM.Application.IpcServer.Server;
using GlazeWM.Application.IpcServer.Messages;
using GlazeWM.Domain.Containers;
using GlazeWM.Domain.Monitors;
using GlazeWM.Domain.UserConfigs;
using GlazeWM.Domain.Windows;
using GlazeWM.Domain.Workspaces;
using GlazeWM.Infrastructure.Bussing;
using GlazeWM.Infrastructure.Serialization;
using GlazeWM.Infrastructure.Utils;
using Microsoft.Extensions.Logging;

namespace GlazeWM.Application.IpcServer
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
        options.Converters.Add(new JsonContainerConverter())
      );

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

        return Parser.Default.ParseArguments<
          InvokeCommandMessage,
          SubscribeMessage,
          GetMonitorsMessage,
          GetWorkspacesMessage,
          GetWindowsMessage
        >(messageParts).MapResult(
          (InvokeCommandMessage message) =>
            HandleInvokeCommandMessage(message, messageString),
          (SubscribeMessage message) =>
            HandleSubscribeMessage(message, sessionId, messageString),
          (GetMonitorsMessage message) =>
            HandleGetMonitorsMessage(message, messageString),
          (GetWorkspacesMessage message) =>
            HandleGetWorkspacesMessage(message, messageString),
          (GetWindowsMessage message) =>
            HandleGetWindowsMessage(message, messageString),
          _ => throw new Exception($"Invalid message '{messageString}'")
        );
      }
      catch (Exception exception)
      {
        return ToResponseMessage(false, null, messageString, exception.Message);
      }
    }

    private string HandleInvokeCommandMessage(
      InvokeCommandMessage message,
      string messageString)
    {
      var contextContainer =
        _containerService.GetContainerById(message.ContextContainerId) ??
        _containerService.FocusedContainer;

      var commandString = CommandParsingService.FormatCommand(message.Command);

      var command = _commandParsingService.ParseCommand(
        commandString,
        contextContainer
      );

      var commandResponse = _bus.Invoke((dynamic)command);
      return ToResponseMessage(commandResponse);
    }

    private string HandleSubscribeMessage(
      SubscribeMessage message,
      Guid sessionId,
      string messageString)
    {
      var eventNames = message.Events.Split(',');

      foreach (var eventName in eventNames)
      {
        if (SubscribedSessions.ContainsKey(eventName))
        {
          var sessionIds = SubscribedSessions.GetValueOrThrow(eventName);
          sessionIds.Add(sessionId);
          continue;
        }

        SubscribedSessions.Add(eventName, new List<Guid>() { sessionId });
      }

      return ToResponseMessage(CommandResponse.Ok);
    }

    private string HandleGetMonitorsMessage(
      GetMonitorsMessage _,
      string messageString)
    {
      var monitors = _monitorService.GetMonitors();
      return ToResponseMessage(true, monitors as IEnumerable<Container>);
    }

    private string HandleGetWorkspacesMessage(
      GetWorkspacesMessage _,
      string messageString)
    {
      var workspaces = _workspaceService.GetActiveWorkspaces();
      return ToResponseMessage(true, workspaces as IEnumerable<Container>);
    }

    private string HandleGetWindowsMessage(
      GetWindowsMessage _,
      string messageString)
    {
      var windows = _windowService.GetWindows();
      return ToResponseMessage(true, windows as IEnumerable<Container>);
    }

    internal string ToResponseMessage<T>(
      bool success,
      T data,
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

      return JsonParser.ToString((dynamic)responseMessage, _serializeOptions);
    }

    internal string ToEventMessage(Event @event)
    {
      var eventMessage = new ServerMessage<Event>(
        Success: true,
        MessageType: ServerMessageType.SubscribedEvent,
        Data: @event,
        Error: null,
        ClientMessage: null
      );

      return JsonParser.ToString((dynamic)eventMessage, _serializeOptions);
    }
  }
}
