# tools/fake_screencast.py
"""A minimal org.gnome.Mutter.ScreenCast service for consumer tests."""
import sys
import gi
gi.require_version('Gio', '2.0')
from gi.repository import Gio, GLib

NODE = int(sys.argv[1])
XML = '''<node>
<interface name="org.gnome.Mutter.ScreenCast"><method name="CreateSession"><arg type="a{sv}" direction="in"/><arg type="o" direction="out"/></method></interface>
<interface name="org.gnome.Mutter.ScreenCast.Session"><method name="RecordMonitor"><arg type="s" direction="in"/><arg type="a{sv}" direction="in"/><arg type="o" direction="out"/></method><method name="Start"/><method name="Stop"/></interface>
<interface name="org.gnome.Mutter.ScreenCast.Stream"><signal name="PipeWireStreamAdded"><arg type="u"/></signal></interface>
</node>'''
info = Gio.DBusNodeInfo.new_for_xml(XML)
loop = GLib.MainLoop()
calls = []

def handle(connection, sender, path, interface, method, params, invocation):
    calls.append(method)
    print(method, flush=True)
    if method == 'CreateSession':
        invocation.return_value(GLib.Variant('(o)', ('/s',)))
    elif method == 'RecordMonitor':
        invocation.return_value(GLib.Variant('(o)', ('/st',)))
    elif method == 'Start':
        invocation.return_value(None)
        connection.emit_signal(None, '/st', 'org.gnome.Mutter.ScreenCast.Stream', 'PipeWireStreamAdded',
                               GLib.Variant('(u)', (NODE,)))
    else:
        invocation.return_value(None)

def on_bus(connection, name):
    for path, iface in (('/org/gnome/Mutter/ScreenCast', 0), ('/s', 1)):
        connection.register_object(path, info.interfaces[iface], handle, None, None)

Gio.bus_own_name(Gio.BusType.SESSION, 'org.gnome.Mutter.ScreenCast', 0, on_bus, None, lambda *a: loop.quit())
loop.run()
