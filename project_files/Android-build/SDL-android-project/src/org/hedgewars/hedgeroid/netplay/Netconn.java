package org.hedgewars.hedgeroid.netplay;

import java.io.File;
import java.io.IOException;

import org.hedgewars.hedgeroid.Utils;
import org.hedgewars.hedgeroid.netplay.JnaFrontlib.IntStrCallback;
import org.hedgewars.hedgeroid.netplay.JnaFrontlib.MetaschemePtr;
import org.hedgewars.hedgeroid.netplay.JnaFrontlib.NetconnPtr;
import org.hedgewars.hedgeroid.netplay.JnaFrontlib.StrCallback;
import org.hedgewars.hedgeroid.netplay.JnaFrontlib.StrStrCallback;

import com.sun.jna.Pointer;

import android.content.Context;
import android.util.Log;

/**
 * Java-wrapper for the C netconn type. Apart from turning netconn into a more Java-like
 * object with methods, this also handles some of the problems of C <-> Java interop (e.g.
 * ensuring that callback objects don't get garbage collected).
 */
public class Netconn {
	private static final JnaFrontlib FLIB = Flib.INSTANCE;
	private static final String DEFAULT_SERVER = "140.247.62.101";
	private static final int DEFAULT_PORT = 46631;
	
	private NetconnPtr conn;
	private String playerName;
	
	public final PlayerList playerList = new PlayerList();
	public final MessageLog lobbyLog;
	public final MessageLog roomLog;
	
	private StrCallback lobbyJoinCb = new StrCallback() {
		public void callback(Pointer context, String arg1) {
			playerList.addPlayer(arg1);
			lobbyLog.appendPlayerJoin(arg1);
		}
	};
	
	private StrStrCallback lobbyLeaveCb = new StrStrCallback() {
		public void callback(Pointer context, String name, String msg) {
			playerList.removePlayer(name);
			lobbyLog.appendPlayerLeave(name, msg);
		}
	};
	
	private StrStrCallback chatCb = new StrStrCallback() {
		public void callback(Pointer context, String name, String msg) {
			getCurrentLog().appendChat(name, msg);
		}
	};
	
	private IntStrCallback messageCb = new IntStrCallback() {
		public void callback(Pointer context, int type, String msg) {
			getCurrentLog().appendMessage(type, msg);
		}
	};
	
	/**
	 * Connect to the official Hedgewars server.
	 * 
	 * @throws IOException if the metascheme file can't be read or the connection to the server fails
	 */
	public Netconn(Context context, String playerName) throws IOException {
		this(context, playerName, DEFAULT_SERVER, DEFAULT_PORT);
	}
	
	/**
	 * Connect to the server with the given hostname and port
	 * 
	 * @throws IOException if the metascheme file can't be read or the connection to the server fails
	 */
	public Netconn(Context context, String playerName, String host, int port) throws IOException {
		if(playerName == null) {
			playerName = "Player";
		}
		this.playerName = playerName;
		this.lobbyLog = new MessageLog(context);
		this.roomLog = new MessageLog(context);
		
		MetaschemePtr meta = null;
		File dataPath = Utils.getDataPathFile(context);
		try {
			String metaschemePath = new File(dataPath, "metasettings.ini").getAbsolutePath();
			meta = FLIB.flib_metascheme_from_ini(metaschemePath);
			if(meta == null) {
				throw new IOException("Missing metascheme");
			}
			conn = FLIB.flib_netconn_create(playerName, meta, dataPath.getAbsolutePath(), host, port);
			if(conn == null) {
				throw new IOException("Unable to connect to the server");
			}
			FLIB.flib_netconn_onLobbyJoin(conn, lobbyJoinCb, null);
			FLIB.flib_netconn_onLobbyLeave(conn, lobbyLeaveCb, null);
			FLIB.flib_netconn_onChat(conn, chatCb, null);
			FLIB.flib_netconn_onMessage(conn, messageCb, null);
		} finally {
			FLIB.flib_metascheme_release(meta);
		}
	}
	
	public void disconnect() {
		if(conn != null) {
			FLIB.flib_netconn_send_quit(conn, "User quit");
			FLIB.flib_netconn_destroy(conn);
			conn = null;
		}
	}
	
	public void tick() {
		FLIB.flib_netconn_tick(conn);
	}
	
	public void sendChat(String s) {
		FLIB.flib_netconn_send_chat(conn, s);
		if(FLIB.flib_netconn_is_in_room_context(conn)) {
			roomLog.appendChat(playerName, s);
		} else {
			lobbyLog.appendChat(playerName, s);
		}
	}
	
	private MessageLog getCurrentLog() {
		if(FLIB.flib_netconn_is_in_room_context(conn)) {
			return roomLog;
		} else {
			return lobbyLog;
		}
	}
	
	public void sendNick(String nick) { FLIB.flib_netconn_send_nick(conn, nick); }
	public void sendPassword(String password) { FLIB.flib_netconn_send_password(conn, password); }
	public void sendQuit(String message) { FLIB.flib_netconn_send_quit(conn, message); }
	
	public boolean isConnected() {
		return conn != null;
	}
	
	@Override
	protected void finalize() throws Throwable {
		if(conn != null) {
			FLIB.flib_netconn_destroy(conn);
			Log.e("Netconn", "Leaked Netconn object");
		}
	}
}