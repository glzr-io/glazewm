using System.Collections.Generic;

namespace LarsWM.Domain.UserConfigs
{
    class UserConfigFileDto
    {
        public GapsConfig Gaps { get; set; }
        public BarConfig Bar { get; set; }
        public List<WorkspaceConfig> Workspaces { get; set; }
        public List<KeybindingsConfig> Keybindings { get; set; }
    }

    struct GapsConfig
    {
        public int InnerGap { get; set; }
        public int OuterGap { get; set; }
    }

    struct BarConfig
    {
        public int Height { get; set; }
    }

    struct WorkspaceConfig
    {
        public string Name { get; set; }
        public string BindToMonitor { get; set; }
        public string CustomDisplayName { get; set; }
        public bool KeepAlive { get; set; }
    }

    struct KeybindingsConfig
    {
        public string Command { get; set; }
        public List<string> Bindings { get; set; }
    }
}
