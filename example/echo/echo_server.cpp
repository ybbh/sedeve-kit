//
// echo_server.cpp
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~
//
// Copyright (c) 2003-2023 Christopher M. Kohlhoff (chris at kohlhoff dot com)
//
// Distributed under the Boost Software License, Version 1.0. (See accompanying
// file LICENSE_1_0.txt or copy at http://www.boost.org/LICENSE_1_0.txt)
//

#define ENABLE_DTM

#include <cstdlib>
#include <iostream>
#include <thread>
#include <utility>
#include <boost/json.hpp>
#include <boost/asio.hpp>
#include <boost/thread/concurrent_queues/sync_queue.hpp>
#include "sedeve_kit.h"
#include "inst_context.h"

using boost::asio::ip::tcp;
using boost::asio::buffer;
using boost::asio::stream_errc::eof;
using boost::system::error_code;
using boost::asio::io_context;
using boost::json::value;
using boost::json::value_to;
using boost::concurrent::sync_queue;

const int max_length = 1024;


void session(
        tcp::socket sock,
        uint64_t local,
        uint64_t remote,
        shared_ptr<sync_queue<message>> queue
    ) {
    try {
        for (;;) {
            char data[max_length];
            error_code error;
            size_t length;

#ifdef ENABLE_DTM
            if (!queue) {
                break;
            }
            auto message = queue->pull();
            if (message.type == "stop") {
                break;
            }
#endif
#ifdef ENABLE_DTM
            {

                if (message.json.length() > max_length) {
                    break;
                }
                value action_receive_from_client = {
                        {"type",    "receive_from_client"},
                        {"message", message.json}
                };
                auto json_receive_from_client = value_to<std::string>(action_receive_from_client);
                INPUT_ACTION(AUTO_ECHO, local, remote, json_receive_from_client.c_str());
            }
#else
            size_t length = sock.read_some(buffer(data), error);
            if (error == eof)
                break; // Connection closed cleanly by peer.
            else if (error)
                throw std::system_error(error); // Some other error.
#endif

#ifdef ENABLE_DTM
            {
                value action_reply_to_client = {
                        {"type",    "reply_to_client"},
                        {"message", message.json}
                };
                auto json_reply_to_client = value_to<std::string>(action_reply_to_client);
                OUTPUT_ACTION(AUTO_ECHO, local, remote, json_reply_to_client.c_str());
            }
#else
            write(sock, buffer(data, length));
#endif
        }
    }
    catch (std::exception &e) {
        std::cerr << "Exception in thread: " << e.what() << "\n";
    }
}

[[noreturn]] void server(io_context &io_context, unsigned short port, uint64_t local_id) {
    INPUT_ACTION(AUTO_ECHO, local_id, local_id, "SERVER_START");
    tcp::acceptor a(io_context, tcp::endpoint(tcp::v4(), port));
    for (;;) {
        tcp::socket sock(io_context);
        a.accept(sock);
        auto endpoint = sock.remote_endpoint();

        uint64_t remote_id =  endpoint_to_id(endpoint.address().to_string() , endpoint.port());
        auto queue = create_sync_queue_for_remote(remote_id);
        std::thread(session, std::move(sock), local_id, remote_id, std::move(queue)).detach();
    }
}

int main(int argc, char *argv[]) {
    try {
        if (argc != 3) {
            std::cerr << "Usage: blocking_tcp_echo_server <port> <id>\n";
            return 1;
        }

        io_context io_context;

        server(io_context, std::atoi(argv[1]), uint64_t(std::atoi(argv[2])));
    }
    catch (std::exception &e) {
        std::cerr << "Exception: " << e.what() << "\n";
    }

    return 0;
}