#include "version.h"

#include "base/file_utils.h"
#include "base/output_file_requirements.h"
#include "era/commands.h"
#include "era/metadata.h"
#include "era/restore_emitter.h"
#include "era/xml_format.h"
#include "persistent-data/file_utils.h"

#include <boost/lexical_cast.hpp>
#include <fstream>
#include <getopt.h>
#include <iostream>
#include <libgen.h>
#include <string>

using namespace boost;
using namespace era;
using namespace persistent_data;
using namespace std;

//----------------------------------------------------------------

namespace {
	struct flags {
		flags()
			: quiet(false) {
		}

		optional<string> input;
		optional<string> output;
		bool quiet;
	};

	int restore(flags const &fs, bool quiet) {
		bool metadata_touched = false;
		try {
			block_manager::ptr bm = open_bm(*fs.output, block_manager::READ_WRITE);
			file_utils::check_file_exists(*fs.input);
			metadata_touched = true;
			metadata::ptr md(new metadata(bm, metadata::CREATE));
			emitter::ptr restorer = create_restore_emitter(*md);

			parse_xml(*fs.input, restorer, fs.quiet);

		} catch (std::exception &e) {
			cerr << e.what() << endl;
			if (metadata_touched)
				try {
					file_utils::zero_superblock(*fs.output);
				} catch (...) {
				}
			return 1;
		}

		return 0;
	}
}

//----------------------------------------------------------------

era_restore_cmd::era_restore_cmd()
	: command("era_restore")
{
}

void
era_restore_cmd::usage(std::ostream &out) const
{
	out << "Usage: " << get_name() << " [options]" << endl
	    << "Options:" << endl
	    << "  {-h|--help}" << endl
	    << "  {-i|--input} <input xml file>" << endl
	    << "  {-o|--output} <output device or file>" << endl
	    << "  {-q|--quiet}" << endl
	    << "  {-V|--version}" << endl;
}

int
era_restore_cmd::run(int argc, char **argv)
{
	int c;
	flags fs;
	char const *short_opts = "hi:o:qV";
	option const long_opts[] = {
		{ "help", no_argument, NULL, 'h'},
		{ "input", required_argument, NULL, 'i' },
		{ "output", required_argument, NULL, 'o'},
		{ "quiet", no_argument, NULL, 'q'},
		{ "version", no_argument, NULL, 'V'},
		{ NULL, no_argument, NULL, 0 }
	};

	while ((c = getopt_long(argc, argv, short_opts, long_opts, NULL)) != -1) {
		switch(c) {
		case 'h':
			usage(cout);
			return 0;

		case 'i':
			fs.input = optional<string>(string(optarg));
			break;

		case 'o':
			fs.output = optional<string>(string(optarg));
			break;

		case 'q':
			fs.quiet = true;
			break;

		case 'V':
			cout << THIN_PROVISIONING_TOOLS_VERSION << endl;
			return 0;

		default:
			usage(cerr);
			return 1;
		}
	}

	if (argc != optind) {
		usage(cerr);
		return 1;
	}

        if (!fs.input) {
		cerr << "No input file provided." << endl << endl;
		usage(cerr);
		return 1;
	}

	if (fs.output)
		check_output_file_requirements(*fs.output);

	else {
		cerr << "No output file provided." << endl << endl;
		usage(cerr);
		return 1;
	}

	return restore(fs, fs.quiet);
}

//----------------------------------------------------------------
