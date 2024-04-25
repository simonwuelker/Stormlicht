use libc::{
    accept, bind, close, cmsghdr, connect, iovec, listen, msghdr, recvmsg, sa_family_t, sendmsg,
    sockaddr, sockaddr_un, socket, socketpair, socklen_t, strncpy, unlink, AF_UNIX, CMSG_DATA,
    CMSG_FIRSTHDR, CMSG_LEN, CMSG_SPACE, SCM_RIGHTS, SOCK_STREAM, SOL_SOCKET,
};
use std::{ffi, io, mem, ptr};

pub struct FileDescriptor(ffi::c_int);

/// A very short-lived ipc server that only serves to
/// share a `fd` between two processes
///
/// This allows us to share descriptors from `socketpair`
/// across unrelated processes, which doesn't expose our ipc
/// internals to the whole world.
pub struct IpcSetupServer {
    fd: FileDescriptor,
}

impl IpcSetupServer {
    const NAME: &'static [u8] = b"stormlicht_ipc_setup_socket\x00";

    #[must_use]
    fn get_sockaddr() -> sockaddr_un {
        let mut socket_address = sockaddr_un {
            sun_family: AF_UNIX as sa_family_t,
            sun_path: [0; 108],
        };

        unsafe {
            strncpy(
                socket_address.sun_path.as_mut_ptr(),
                Self::NAME.as_ptr() as *const ffi::c_char,
                mem::size_of_val(&socket_address.sun_path) - 1,
            );
        }

        socket_address
    }

    fn init_socket() -> Result<ffi::c_int, io::Error> {
        // Open the socket
        let fd = unsafe { socket(AF_UNIX, SOCK_STREAM, 0) };
        if fd == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(fd)
    }

    pub fn create() -> Result<Self, io::Error> {
        let fd = Self::init_socket()?;

        // Bind it to the given address
        let socket_address = Self::get_sockaddr();
        let status = unsafe {
            bind(
                fd,
                &socket_address as *const _ as *const sockaddr,
                mem::size_of_val(&socket_address) as socklen_t,
            )
        };
        if status == -1 {
            log::error!("Failed to bind socket");
            return Err(io::Error::last_os_error());
        }

        // Arbitrarily chose 5 as backlog size
        let status = unsafe { listen(fd, 5) };
        if status == -1 {
            log::error!("Failed to listen on socket");
            return Err(io::Error::last_os_error());
        }

        let server = Self {
            fd: FileDescriptor(fd),
        };

        Ok(server)
    }

    /// Accept an incoming connection
    pub fn accept(&self) -> Result<IpcClient, io::Error> {
        let client_fd = unsafe { accept(self.fd.0, std::ptr::null_mut(), std::ptr::null_mut()) };

        if client_fd == -1 {
            log::error!("Failed to accept connection");
            return Err(io::Error::last_os_error());
        }

        let client = IpcClient { fd: client_fd };

        Ok(client)
    }
}

pub struct IpcClient {
    fd: ffi::c_int,
}

impl IpcClient {
    pub fn pair() -> io::Result<(Self, Self)> {
        let mut fds = [0; 2];
        let status = unsafe {
            socketpair(
                AF_UNIX,
                SOCK_STREAM,
                0,
                &mut fds as *mut _ as *mut ffi::c_int,
            )
        };
        if status == -1 {
            log::error!("Failed create connected sockets");
            return Err(io::Error::last_os_error());
        }

        let first = Self { fd: fds[0] };
        let second = Self { fd: fds[1] };

        Ok((first, second))
    }

    pub fn connect() -> io::Result<Self> {
        let fd = IpcSetupServer::init_socket()?;
        let addr = IpcSetupServer::get_sockaddr();

        let status = unsafe {
            connect(
                fd,
                &addr as *const _ as *const sockaddr,
                mem::size_of_val(&addr) as socklen_t,
            )
        };
        if status == -1 {
            return Err(io::Error::last_os_error());
        }

        let client = Self { fd };

        Ok(client)
    }

    pub fn send_bytes(&self, bytes: &mut [u8]) -> io::Result<()> {
        let mut io_vec = iovec {
            iov_base: bytes.as_mut_ptr() as *mut _ as *mut ffi::c_void,
            iov_len: bytes.len(),
        };

        let message = msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut io_vec,
            msg_iovlen: 1,
            msg_control: ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        let status = unsafe { sendmsg(self.fd, &message, 0) };
        if status == -1 {
            log::error!("Failed to send bytes");
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn recv_bytes(&self, buf: &mut [u8]) -> io::Result<()> {
        let mut io_vec = iovec {
            iov_base: buf.as_mut_ptr() as *mut _ as *mut ffi::c_void,
            iov_len: buf.len(),
        };

        let mut message = msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut io_vec,
            msg_iovlen: 1,
            msg_control: ptr::null_mut(),
            msg_controllen: 0,
            msg_flags: 0,
        };

        let status = unsafe { recvmsg(self.fd, &mut message, 0) };
        if status == -1 {
            log::error!("Failed to read bytes from socket");
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn send_fd(&self, fd: ffi::c_int) -> io::Result<()> {
        // We're only sending control messages, not regular data
        let mut io_vec = iovec {
            iov_base: ptr::null_mut(),
            iov_len: 0,
        };

        const CMSG_LENGTH: u32 = mem::size_of::<ffi::c_int>() as u32;
        const SPACE_REQUIRED: u32 = unsafe { CMSG_SPACE(CMSG_LENGTH) };
        let control_message = &mut [0; SPACE_REQUIRED as usize] as *mut _ as *mut cmsghdr;

        unsafe {
            (*control_message).cmsg_len = CMSG_LEN(CMSG_LENGTH) as usize;
            (*control_message).cmsg_level = SOL_SOCKET;
            (*control_message).cmsg_type = SCM_RIGHTS;
            (CMSG_DATA(control_message) as *mut ffi::c_int).write(fd);
        }

        let message = msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: &mut io_vec,
            msg_iovlen: 1,
            msg_control: control_message as *mut ffi::c_void,
            msg_controllen: SPACE_REQUIRED as usize,
            msg_flags: 0,
        };

        let status = unsafe { sendmsg(self.fd, &message, 0) };
        if status == -1 {
            log::error!("Failed to send fd");
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }

    pub fn receive_fd(&self) -> io::Result<FileDescriptor> {
        const SPACE_REQUIRED: u32 = unsafe { CMSG_SPACE(mem::size_of::<ffi::c_int>() as u32) };
        let mut cmsg_buf = [0_u8; SPACE_REQUIRED as usize];

        let mut header = msghdr {
            msg_name: ptr::null_mut(),
            msg_namelen: 0,
            msg_iov: ptr::null_mut(),
            msg_iovlen: 0,
            msg_control: &mut cmsg_buf as *mut _ as *mut ffi::c_void,
            msg_controllen: SPACE_REQUIRED as usize,
            msg_flags: 0,
        };
        let status = unsafe { recvmsg(self.fd, &mut header, 0) };
        if status == -1 {
            log::error!("Failed to receive fd");
            return Err(io::Error::last_os_error());
        }

        let cmsg = unsafe { CMSG_FIRSTHDR(&header) };
        let fd = unsafe { *(CMSG_DATA(cmsg) as *const ffi::c_int) };

        Ok(FileDescriptor(fd))
    }
}

impl Drop for IpcSetupServer {
    fn drop(&mut self) {
        let status = unsafe { close(self.fd.0) };

        if status != 0 {
            panic!(
                "Failed to close IpcSetupServer: {:?}",
                io::Error::last_os_error()
            )
        }

        let status = unsafe { unlink(Self::NAME.as_ptr() as *const ffi::c_char) };
        if status != 0 {
            panic!(
                "Failed to unlink IpcSetupServer socket: {:?}",
                io::Error::last_os_error()
            )
        }
    }
}

impl Drop for IpcClient {
    fn drop(&mut self) {
        let status = unsafe { close(self.fd) };

        if status != 0 {
            panic!(
                "Failed to close IpcClient: {:?}",
                io::Error::last_os_error()
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_client() {
        let (a, b) = IpcClient::pair().unwrap();

        let mut send_buf = [1, 2, 3];
        let mut recv_buf = [0; 3];

        a.send_bytes(&mut send_buf).unwrap();
        b.recv_bytes(&mut recv_buf).unwrap();

        assert_eq!(send_buf, recv_buf);
    }
}
